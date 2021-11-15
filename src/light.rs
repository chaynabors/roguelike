use serde::Deserialize;
use serde::Serialize;

use crate::color::Color;
use crate::vector2::Vector2;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Light {
    position: Vector2,
    color: Color,
    luminosity: f32,
}
