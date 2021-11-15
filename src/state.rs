use serde::Deserialize;
use serde::Serialize;

use crate::camera::Camera;
use crate::map::Map;
use crate::player::Player;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State{
    map: Map,
    camera: Camera,
    player: Player,
}