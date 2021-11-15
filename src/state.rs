use crate::map::Map;
use crate::camera::Camera;
use crate::player::Player;

#[derive(Clone, Debug)]
pub struct State{
    map: Map,
    camera: Camera,
    player: Player,
}