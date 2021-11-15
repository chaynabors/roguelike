use serde::Deserialize;
use serde::Serialize;

use crate::vector2::Vector2;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Player {
    pub position: Vector2,
}
