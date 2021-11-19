use serde::Deserialize;
use serde::Serialize;

use crate::camera::Camera;
use crate::map::Map;
use crate::player::Player;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct State {
    map: Map,
    camera: Camera,
    player: Player,
}

#[cfg(test)]
mod tests {
    use super::State;

    #[test]
    fn serialization() {
        let state = State::default();
        let serialized = serde_json::to_string_pretty(&state).unwrap();    
        serde_json::from_str::<State>(&serialized).unwrap();
    }
}
