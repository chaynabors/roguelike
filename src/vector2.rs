use serde::Deserialize;
use serde::Serialize;

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[cfg(test)]
mod tests {
    use super::Vector2;

    #[test]
    fn serialization() {
        let vector2 = Vector2::default();
        let serialized = serde_json::to_string(&vector2).unwrap();
        serde_json::from_str::<Vector2>(&serialized).unwrap();
    }
}
