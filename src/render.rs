use screen_13::prelude::*;
use winit::{
    event_loop::ActiveEventLoop,
    window::Window
};
use anyhow::{
    Result,
    bail,
};
#[allow(unused)]
use log::{
    trace,
    debug,
    warn,
    error,
};
use image::RgbaImage;
use std::sync::Arc;
use crate::{
    math::*,
    Uuid,
    IdMap,
    Color,
};


pub enum Shape2D {
    /// A list of points where each point connects to the next one to form a line
    Line(Vec<Color>, Vec<Point2>),
    ColorPolygon {
        colors: Vec<Color>,
        /// The vertex positions for the triangles
        vertices: Vec<Point2>,
        /// A list of indices for each triangle. Length should be a multiple of 3.
        indices: Vec<u16>,
    },
    TexturePolygon {
        texture: ImageID,
        /// The UV for each vertex
        uvs: Vec<Point2>,
        /// The vertex positions for the triangles
        vertices: Vec<Point2>,
        /// A list of indices for each triangle. Length should be a multiple of 3.
        indices: Vec<u16>,
    },
}

/// TODO(3d shapes): Implement all the things to render 3d shapes
pub enum Shape3 {
    ColorPolygon {
        colors: Vec<Color>,
        /// The vertex positions for the triangles
        vertices: Vec<Point3>,
        /// A list of indices for each triangle. Length should be a multiple of 3.
        indices: Vec<u16>,
    },
    TexturePolygon {
        texture: Uuid,
        /// The UV for each vertex
        uvs: Vec<Point2>,
        /// The vertex positions for the triangles
        vertices: Vec<Point3>,
        /// A list of indices for each triangle. Length should be a multiple of 3.
        indices: Vec<u16>,
    },
}

pub enum Shape2DInternal {
    Line(Arc<Buffer>, u32),
    ColorPoly {
        vertex_color: Arc<Buffer>,
        index_count: u32,
        index: Arc<Buffer>,
    },
    TexturePoly {
        vert_uv: Arc<Buffer>,
        index_count: u32,
        index: Arc<Buffer>,
        texture: Arc<Image>,
    },
}


pub struct State2D {
    pub line: Arc<GraphicPipeline>,
    pub color_poly: Arc<GraphicPipeline>,
    pub tex_poly: Arc<GraphicPipeline>,

    pub shapes: IdMap<(Shape2DInternal, Shape2D)>,
}
impl State2D {
    pub fn new(device: &Arc<Device>)->Result<Self> {
        let line = line_shaders()?
            .line_pipeline(&device)?;
        let color_poly = color_poly2_shaders()?
            .polygon_pipeline(&device)?;
        let tex_poly = tex_poly2_shaders()?
            .polygon_pipeline(&device)?;

        return Ok(State2D {
            line: Arc::new(line),
            color_poly: Arc::new(color_poly),
            tex_poly: Arc::new(tex_poly),
            shapes: IdMap::default(),
        });
    }
}

pub struct Renderer {
    /// Only supports 32bit RGBA-sRGB 2D images
    pub images: IdMap<Arc<Image>>,

    /// Data to process 2D shapes
    pub d2: State2D,

    pub display: Display,
    pub display_pool: HashPool,
    pub device: Arc<Device>,
    pub window: Arc<Window>,
}
impl Renderer {
    pub const DEFAULT_IMG_FORMAT: vk::Format = vk::Format::R8G8B8A8_SRGB;
    pub const DEFAULT_CLR_SPACE: vk::ColorSpaceKHR = vk::ColorSpaceKHR::SRGB_NONLINEAR;


    pub fn new(el: &ActiveEventLoop, window_title: impl Into<String>)->Result<Self> {
        trace!("New renderer");
        let mut attrs = Window::default_attributes();
        attrs.title = window_title.into();

        let window = Arc::new(el.create_window(attrs)?);
        let win_size = window.inner_size();

        let mut device_info = DeviceInfo::default();
        device_info.debug = true;

        let device = Arc::new(Device::create_display(device_info, &window)?);
        let surface = Surface::create(&device,&window)?;
        let mut sci = SwapchainInfo::new(
            win_size.width,
            win_size.height,
            vk::SurfaceFormatKHR {
                format: Self::DEFAULT_IMG_FORMAT,
                color_space: Self::DEFAULT_CLR_SPACE,
            },
        );
        sci.sync_display = true;
        sci.desired_image_count = 2;

        let swapchain = Swapchain::new(&device, surface, sci)?;
        let display = Display::new(&device, swapchain, DisplayInfo::default())?;

        let d2 = State2D::new(&device)?;

        return Ok(Renderer {
            display_pool: HashPool::new(&device),
            window,
            device,
            display,

            d2,

            images: IdMap::default(),
        });
    }

    pub fn add_shape2d(&mut self, shape: Shape2D)->Result<ShapeID> {
        let id = crate::new_uuid();
        match &shape {
            Shape2D::Line(colors, points)=>{
                let vertex_color = points.iter().copied()
                    .zip(colors.iter().copied())
                    .map(|(v,col)|[v.x,v.y,col.r,col.g,col.b,col.a].into_iter())
                    .flatten()
                    .collect::<Vec<f32>>();
                let vertex_color = Arc::new(Buffer::create_from_slice(
                    &self.device,
                    vk::BufferUsageFlags::VERTEX_BUFFER,
                    bytemuck::cast_slice(vertex_color.as_slice()),
                )?);
                let shape_internal = Shape2DInternal::Line(vertex_color, points.len() as u32);

                self.d2.shapes.insert(id, (shape_internal, shape));
            },
            Shape2D::ColorPolygon{colors, vertices, indices}=>{
                let vertex_color = vertices.iter().copied()
                    .zip(colors.iter().copied())
                    .map(|(v,col)|[v.x,v.y,col.r,col.g,col.b,col.a].into_iter())
                    .flatten()
                    .collect::<Vec<f32>>();
                let index = Arc::new(Buffer::create_from_slice(
                    &self.device,
                    vk::BufferUsageFlags::INDEX_BUFFER,
                    bytemuck::cast_slice(indices.as_slice()),
                )?);
                let vertex_color = Arc::new(Buffer::create_from_slice(
                    &self.device,
                    vk::BufferUsageFlags::VERTEX_BUFFER,
                    bytemuck::cast_slice(vertex_color.as_slice()),
                )?);
                let shape_internal = Shape2DInternal::ColorPoly {
                    index,
                    vertex_color,
                    index_count: indices.len() as u32,
                };

                self.d2.shapes.insert(id, (shape_internal, shape));
            },
            Shape2D::TexturePolygon{texture, vertices, uvs, indices}=>{
                let Some(texture) = self.images.get(&texture.0) else {bail!("Texture does not exist")};
                let vert_uv = vertices.iter().copied()
                    .zip(uvs.iter().copied())
                    .map(|(v,uv)|[v.x,v.y,uv.x,uv.y].into_iter())
                    .flatten()
                    .collect::<Vec<f32>>();
                let index = Arc::new(Buffer::create_from_slice(
                    &self.device,
                    vk::BufferUsageFlags::INDEX_BUFFER,
                    bytemuck::cast_slice(indices.as_slice()),
                )?);
                let vert_uv = Arc::new(Buffer::create_from_slice(
                    &self.device,
                    vk::BufferUsageFlags::VERTEX_BUFFER,
                    bytemuck::cast_slice(vert_uv.as_slice()),
                )?);
                let shape_internal = Shape2DInternal::TexturePoly {
                    index,
                    vert_uv,
                    index_count: indices.len() as u32,
                    texture: texture.clone(),
                };

                self.d2.shapes.insert(id, (shape_internal, shape));
            },
        }

        trace!("Added shape with id: {id}");
        return Ok(ShapeID(id));
    }

    pub fn drop_image(&mut self, id: ImageID) {
        self.images.remove(&id.0);
    }

    pub fn drop_shape2d(&mut self, id: ShapeID) {
        self.d2.shapes.remove(&id.0);
    }

    #[inline]
    pub fn begin<'render>(&'render mut self)->Result<RenderFrame<'render>> {
        let mut graph = RenderGraph::new();

        let Some(sc_img) = self.display.acquire_next_image()? else {
            bail!("Could not get swapchain image!");
        };
        let sc_node = graph.bind_node(sc_img);

        graph.clear_color_image(sc_node);
        trace!("Start a render pass");

        return Ok(RenderFrame {
            graph,
            swapchain_node: sc_node,
            renderer: self,

            line_count: 0,
            clr_poly_count: 0,
            tex_poly_count: 0,
        });
    }

    pub fn on_resize_event(&mut self) {
        let size = self.window.inner_size();
        let mut sc_info = self.display.swapchain_info();
        sc_info.width = size.width;
        sc_info.height = size.height;
        self.display.set_swapchain_info(sc_info);
    }

    #[inline]
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn upload_image(&mut self, img: RgbaImage)->Result<ImageID> {
        let mut graph = RenderGraph::new();

        let id = self.upload_image_with_graph(&mut graph, img)?;

        graph.resolve().submit(&mut FifoPool::new(&self.device), 0, 0)?;

        return Ok(id);
    }

    pub fn upload_image_with_graph(&mut self, graph: &mut RenderGraph, img: RgbaImage)->Result<ImageID> {
        let img_raw = img.as_raw().as_slice();

        let buf_flags = vk::BufferUsageFlags::TRANSFER_SRC|vk::BufferUsageFlags::TRANSFER_DST;

        let buf = Buffer::create_from_slice(&self.device, buf_flags, &img_raw)?;

        let id = crate::new_uuid();

        let img_info = ImageInfo::image_2d(
            img.width(),
            img.height(),
            Renderer::DEFAULT_IMG_FORMAT,
            vk::ImageUsageFlags::TRANSFER_DST|vk::ImageUsageFlags::SAMPLED|vk::ImageUsageFlags::COLOR_ATTACHMENT,
        );
        let img = Arc::new(Image::create(&self.device, img_info)?);
        let img_node = graph.bind_node(&img);
        let buf = graph.bind_node(buf);
        graph.copy_buffer_to_image(buf, img_node);
        self.images.insert(id, img);
        return Ok(ImageID(id));
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ImageID(pub Uuid);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ShapeID(pub Uuid);

pub struct RenderFrame<'render> {
    pub renderer: &'render mut Renderer,
    pub graph: RenderGraph,
    pub swapchain_node: SwapchainImageNode,

    pub line_count: usize,
    pub clr_poly_count: usize,
    pub tex_poly_count: usize,
}
impl<'render> RenderFrame<'render> {
    pub fn custom(&mut self, render_fn: fn(&mut Renderer, &mut RenderGraph))->&mut Self {
        render_fn(self.renderer, &mut self.graph);

        return self;
    }

    #[inline]
    pub fn upload_image(&mut self, img: RgbaImage)->Result<ImageID> {
        self.renderer.upload_image_with_graph(&mut self.graph, img)
    }

    pub fn shape2d(&mut self, id: ShapeID, transform: Transform2)->Result<&mut Self> {
        let Some((shape, _)) = self.renderer.d2.shapes.get(&id.0) else {bail!("Shape with ID `{id:?}` not found")};
        let transform = transform.into_homogeneous_matrix();

        let columns = transform.as_component_array();

        let mut bytes: Vec<u8> = Vec::with_capacity(64);
        bytes.extend(columns[0].as_byte_slice());
        bytes.extend([0;4]);
        bytes.extend(columns[1].as_byte_slice());
        bytes.extend([0;4]);
        bytes.extend(columns[2].as_byte_slice());
        bytes.extend([0;4]);

        match shape {
            Shape2DInternal::Line(vertex_color, vertex_count)=>{
                trace!("Render a line");
                self.line_count += 1;
                let mut pass = self.graph
                    .begin_pass(format!("Line #{}", self.line_count))
                    .bind_pipeline(&self.renderer.d2.line);
                let vertex_count = *vertex_count;
                let cv_node = pass.bind_node(vertex_color);
                pass
                    .access_node(cv_node, AccessType::VertexBuffer)
                    .store_color(0, self.swapchain_node)
                    .record_subpass(move|sp, _|{
                        sp.push_constants(&bytes);
                        sp.bind_vertex_buffer(cv_node);
                        sp.draw(vertex_count, 1, 0, 0);
                    })
                    .submit_pass();
            },
            Shape2DInternal::ColorPoly{vertex_color, index_count, index}=>{
                trace!("Render a colored polygon");
                self.clr_poly_count += 1;
                let mut pass = self.graph
                    .begin_pass(format!("ColorPoly #{}", self.clr_poly_count))
                    .bind_pipeline(&self.renderer.d2.color_poly);
                let index_count = *index_count;
                let cv_node = pass.bind_node(vertex_color);
                let index_node = pass.bind_node(index);
                pass
                    .access_node(cv_node, AccessType::VertexBuffer)
                    .access_node(index_node, AccessType::IndexBuffer)
                    .store_color(0, self.swapchain_node)
                    .record_subpass(move|sp, _|{
                        sp.push_constants(&bytes);
                        sp.bind_vertex_buffer(cv_node);
                        sp.bind_index_buffer(index_node, vk::IndexType::UINT16);
                        sp.draw_indexed(index_count, 1, 0, 0, 0);
                    })
                    .submit_pass();
            },
            Shape2DInternal::TexturePoly{vert_uv: vertex_uv, index_count, index, texture}=>{
                trace!("Render a textured polygon");
                self.tex_poly_count += 1;
                let mut pass = self.graph
                    .begin_pass(format!("TexturePoly #{}", self.tex_poly_count))
                    .bind_pipeline(&self.renderer.d2.tex_poly);
                let index_count = *index_count;
                let cv_node = pass.bind_node(vertex_uv);
                let index_node = pass.bind_node(index);
                let texture_node = pass.bind_node(texture);
                pass
                    .access_node(cv_node, AccessType::VertexBuffer)
                    .access_node(index_node, AccessType::IndexBuffer)
                    .read_descriptor(0, texture_node)
                    .store_color(0, self.swapchain_node)
                    .record_subpass(move|sp, _|{
                        sp.push_constants(&bytes);
                        sp.bind_vertex_buffer(cv_node);
                        sp.bind_index_buffer(index_node, vk::IndexType::UINT16);
                        sp.draw_indexed(index_count, 1, 0, 0, 0);
                    })
                    .submit_pass();
            },
        }

        return Ok(self);
    }

    /// Finish the render and submit it to the GPU
    pub fn finish(self)->Result<()> {
        trace!("Finish rendering");
        self.renderer.window.pre_present_notify();
        self.renderer.display.present_image(
            &mut self.renderer.display_pool,
            self.graph,
            self.swapchain_node,
            0,
        )?;
        self.renderer.window.request_redraw();

        return Ok(());
    }
}

pub struct ShaderInternal {
    pub vert: Shader,
    pub frag: Shader,
}
impl ShaderInternal {
    /// A line strip pipeline
    pub fn line_pipeline(self, device: &Arc<Device>)->Result<GraphicPipeline> {
        self.pipeline(
            device,
            GraphicPipelineInfo::builder()
                .topology(vk::PrimitiveTopology::LINE_STRIP)
                .polygon_mode(vk::PolygonMode::LINE),
        )
    }

    /// A filled triangle list pipeline
    pub fn polygon_pipeline(self, device: &Arc<Device>)->Result<GraphicPipeline> {
        self.pipeline(
            device,
            GraphicPipelineInfo::builder()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .polygon_mode(vk::PolygonMode::FILL),
        )
    }

    /// A new custom pipeline
    pub fn pipeline(self, device: &Arc<Device>, info: impl Into<GraphicPipelineInfo>)->Result<GraphicPipeline> {
        Ok(GraphicPipeline::create(
            device,
            info,
            [self.vert, self.frag],
        )?)
    }
}


fn line_shaders()->Result<ShaderInternal> {
    translate_shaders(
        "shaders/line_vert.glsl",
        "shaders/line_frag.glsl",
    )
}

fn color_poly2_shaders()->Result<ShaderInternal> {
    translate_shaders(
        "shaders/color_poly2_vert.glsl",
        "shaders/color_poly2_frag.glsl",
    )
}

fn tex_poly2_shaders()->Result<ShaderInternal> {
    translate_shaders(
        "shaders/tex_poly2_vert.glsl",
        "shaders/tex_poly2_frag.glsl",
    )
}

/// Translates WGSL shader text to SPIR-V binary data for use in a `GraphicsPipeline`
pub fn translate_shaders(vert_path: &str, frag_path: &str)->Result<ShaderInternal> {
    use shaderc::{
        Compiler,
        ShaderKind,
    };

    let vert_text = std::fs::read_to_string(vert_path)?;
    let frag_text = std::fs::read_to_string(frag_path)?;

    let compiler = Compiler::new()?;
    let vert = compiler.compile_into_spirv(
        &vert_text,
        ShaderKind::Vertex,
        vert_path,
        "main",
        None,
    )?;
    let frag = compiler.compile_into_spirv(
        &frag_text,
        ShaderKind::Fragment,
        frag_path,
        "main",
        None,
    )?;

    let vert = Shader::new_vertex(vert.as_binary()).build();
    let frag = Shader::new_fragment(frag.as_binary()).build();

    return Ok(ShaderInternal {vert, frag});
}
