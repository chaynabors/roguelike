use crate::color::Color;
use crate::vector2::Vector2;

#[derive(Copy, Clone, Debug)]
pub struct Light {
    position: Vector2,
    color: Color,
    luminosity: f32,
}
