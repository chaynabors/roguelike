use serde::Deserialize;
use serde::Serialize;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}
