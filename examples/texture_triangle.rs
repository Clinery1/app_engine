use winit::{
    event_loop::{
        ActiveEventLoop,
        EventLoopProxy,
    },
    window::WindowId,
    event::WindowEvent,
};
use anyhow::Result;
use app_engine::{
    render::*,
    math::*,
    AppEngine,
    App,
    load_image,
};
use std::time::Instant;


struct TriangleExample {
    render: Renderer,
    shape: ShapeID,
    transform: Transform2,
    move_up: bool,
    last_frame: Instant,
}
impl TriangleExample {
    fn render(&mut self)->Result<()> {
        let last_frame = std::mem::replace(&mut self.last_frame, Instant::now());
        let duration = last_frame.elapsed();
        log::debug!("Frame time: {duration:?}");
        let dt = duration.as_secs_f32();
        if self.move_up {
            self.transform.translation.y += 0.5 * dt;
            if self.transform.translation.y >= 0.0 {
                self.move_up = false;
            }
        } else {
            self.transform.translation.y -= 0.5 * dt;
            if self.transform.translation.y <= -1.0 {
                self.move_up = true;
            }
        }
        let mut frame = self.render.begin()?;
        frame.shape2d(self.shape, self.transform)?;
        frame.finish()?;
        return Ok(());
    }
}
impl App for TriangleExample {
    fn new(el: &ActiveEventLoop, _: EventLoopProxy<()>)->Result<Self> {
        let mut render = Renderer::new(el, "Triangle Example")?;
        let image = load_image("sample.png")?;
        let texture = render.upload_image(image)?;
        let shape = render.add_shape2d(Shape2D::TexturePolygon {
            texture,
            indices: vec![0, 1, 2, 3, 2, 1],
            vertices: vec![
                Point2::new(0.0, 0.0),
                Point2::new(0.0, 1.0),
                Point2::new(1.0, 0.0),
                Point2::new(1.0, 1.0),
            ],
            uvs: vec![
                Point2::new(0.0, 0.0),
                Point2::new(0.0, 1.0),
                Point2::new(1.0, 0.0),
                Point2::new(1.0, 1.0),
            ],
        })?;
        return Ok(TriangleExample {
            render,
            shape,
            transform: Transform2::new(Point2::new(0.0, 0.0), Rotation2::from_angle(0.0), 1.0),
            move_up: false,
            last_frame: Instant::now(),
        });
    }
    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, ev: WindowEvent) {
        match ev {
            WindowEvent::CloseRequested=>el.exit(),
            WindowEvent::Resized(_)=>self.render.on_resize_event(),
            WindowEvent::RedrawRequested=>{
                match self.render() {
                    Ok(())=>{},
                    Err(e)=>{
                        el.exit();
                        eprintln!("Error rendering: {e}");
                    },
                }
            },
            _=>{},
        }
    }
    fn resumed(&mut self, _el: &ActiveEventLoop) {}
}


fn main() {
    simplelog::TermLogger::init(
        log::LevelFilter::Debug,
        simplelog::ConfigBuilder::new()
            .set_location_level(log::LevelFilter::Off)
            .build(),
        simplelog::TerminalMode::Stderr,
        simplelog::ColorChoice::Always,
    ).unwrap();

    AppEngine::<(), TriangleExample>::run()
        .unwrap();
}
