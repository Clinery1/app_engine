//! The UI module contains a simple layout system that works with screen pixels.
//! The system is immediate, so it does the layout each frame.


pub mod text;
pub mod button;
pub mod container;


pub enum Size {
    Weight(u32),
    Pixels(u32),
    Fill,
}
