//! # About
//! This is a very opinionated base for a simple 2D GUI application.
//!
//! # List of features
//! - UI module
//! - Simple render module for 2D applications with the option to write a custom 3d rendering
//!     pipeline
//! - Allows access to internals of the renderer, ui state, etc. for custom behavior. This might
//!     be seen as an abuse of `pub`, but this crate is not being designed for an end-user, but
//!     instead to be dog-fooded into my other projects, like games. Games usually require a bit
//!     more than just "render a shape with a color" or "render a shape with a texture," so some
//!     access to the "internals" of the renderer are a good thing.


pub use uuid::Uuid;
pub use anyhow::Result;
use winit::{
    event_loop::{
        EventLoop,
        ActiveEventLoop,
        EventLoopProxy,
    },
    window::WindowId,
    event::{
        DeviceId,
        DeviceEvent,
        WindowEvent,
        StartCause,
    },
    application::ApplicationHandler,
};
use image::{
    RgbaImage,
    ImageReader,
};
use std::marker::PhantomData;


pub mod render;
pub mod ui;

pub mod math {
    pub use ultraviolet as uv;

    pub use uv::{
        Vec2,
        Vec2 as Translation2,
        Vec2 as Point2,
        Rotor2 as Rotation2,
        Similarity2 as Transform2,
    };
    pub use uv::{
        Vec3,
        Vec3 as Translation3,
        Vec3 as Point3,
        Rotor3 as Rotation3,
        Similarity3 as Transform3,
    };
    pub use uv::{
        Mat2,
        Mat3,
        Mat4,
    };
}


pub type IdMap<T> = fnv::FnvHashMap<Uuid, T>;
pub type IdSet = fnv::FnvHashSet<Uuid>;

/// This is a hopefully easier way of creating an application.
/// See [`winit::application::ApplicationHandler`] for more information.
pub trait App<T: 'static = ()>: Sized {
    fn new(el: &ActiveEventLoop, proxy: EventLoopProxy<T>)->Result<Self>;
    fn resumed(&mut self, el: &ActiveEventLoop);
    fn window_event(&mut self, el: &ActiveEventLoop, id: WindowId, ev: WindowEvent);

    fn custom_event(&mut self, _el: &ActiveEventLoop, _ev: T) {}
    fn device_event(&mut self, _el: &ActiveEventLoop, _id: DeviceId, _de: DeviceEvent) {}
}


#[repr(C,align(4))]
#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Debug)]
pub struct AppEngine<EV: 'static, T: App<EV>> {
    pub app: Option<T>,
    pub proxy: EventLoopProxy<EV>,
    _phantom: PhantomData<EV>,
}
impl<EV: 'static, T: App<EV>> AppEngine<EV, T> {
    pub fn run_init(app: T)->Result<()> {
        let el = EventLoop::with_user_event().build()?;

        let mut init = Self {
            app: Some(app),
            proxy: el.create_proxy(),
            _phantom: PhantomData,
        };

        el.run_app(&mut init)?;

        return Ok(());
    }

    pub fn run()->Result<()> {
        let el = EventLoop::with_user_event().build()?;

        let mut init = Self {
            app: None,
            proxy: el.create_proxy(),
            _phantom: PhantomData,
        };

        el.run_app(&mut init)?;

        return Ok(());
    }
}
impl<EV, T: App<EV>> ApplicationHandler<EV> for AppEngine<EV, T> {
    // Initialize the app
    fn new_events(&mut self, el: &ActiveEventLoop, cause: StartCause) {
        match cause {
            StartCause::Init if self.app.is_none()=>match T::new(el, self.proxy.clone()) {
                Ok(app)=>self.app = Some(app),
                Err(e)=>{
                    eprintln!("Error: {e:#}");
                    eprintln!("{}", e.backtrace());
                    el.exit();
                },
            },
            _=>{},
        }
    }
    fn resumed(&mut self, el: &ActiveEventLoop) {
        self.app
            .as_mut()
            .unwrap()
            .resumed(el);
    }
    fn window_event(&mut self, el: &ActiveEventLoop, id: WindowId, ev: WindowEvent) {
        self.app
            .as_mut()
            .unwrap()
            .window_event(el, id, ev);
    }

    fn device_event(&mut self, el: &ActiveEventLoop, id: DeviceId, ev: DeviceEvent) {
        self.app
            .as_mut()
            .unwrap()
            .device_event(el, id, ev);
    }

    fn user_event(&mut self, el: &ActiveEventLoop, ev: EV) {
        self.app
            .as_mut()
            .unwrap()
            .custom_event(el, ev);
    }
}


#[allow(non_snake_case)]
pub fn Color(r: f32, g: f32, b: f32, a:f32)->Color {
    Color {r, g, b,a}
}

pub fn new_uuid()->Uuid {
    Uuid::new_v4()
}

pub fn load_image(path: impl AsRef<std::path::Path>)->Result<RgbaImage> {
    let decoded = ImageReader::open(path)?
        .decode()?;
    return Ok(decoded.to_rgba8());
}
