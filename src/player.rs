use serde::Deserialize;
use serde::Serialize;

use crate::vector2::Vector2;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub position: Vector2,
}
