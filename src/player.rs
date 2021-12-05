use serde::Deserialize;
use serde::Serialize;

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Player {
    pub position: [f32; 2],
}

#[cfg(test)]
mod tests {
    use super::Player;

    #[test]
    fn serialization() {
        let player = Player::default();
        let serialized = serde_json::to_string(&player).unwrap();
        serde_json::from_str::<Player>(&serialized).unwrap();
    }
}
