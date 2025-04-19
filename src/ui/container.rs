use crate::Color;
use super::Size;


pub struct Container {
    pub width: Size,
    pub height: Size,
}

pub struct ContainerStyle {
    pub border: Option<Color>,
    pub bg: Option<Color>,
}
