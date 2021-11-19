use serde::Deserialize;
use serde::Serialize;

use crate::vector2::Vector2;

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Player {
    pub position: Vector2,
}

#[cfg(test)]
mod tests {
    use super::Player;

    #[test]
    fn serialization() {
        let player = Player::default();
        let serialized = serde_json::to_string_pretty(&player).unwrap();    
        serde_json::from_str::<Player>(&serialized).unwrap();
    }
}
