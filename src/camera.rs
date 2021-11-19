use serde::Deserialize;
use serde::Serialize;

use crate::vector2::Vector2;

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Camera {
    position: Vector2,
}

#[cfg(test)]
mod tests {
    use super::Camera;

    #[test]
    fn serialization() {
        let camera = Camera::default();
        let serialized = serde_json::to_string(&camera).unwrap();
        serde_json::from_str::<Camera>(&serialized).unwrap();
    }
}
