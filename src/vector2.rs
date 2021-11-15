use serde::Deserialize;
use serde::Serialize;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}
