use serde::Deserialize;
use serde::Serialize;

use crate::color::Color;
use crate::vector2::Vector2;

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Light {
    position: Vector2,
    color: Color,
    luminosity: f32,
}

#[cfg(test)]
mod tests {
    use super::Light;

    #[test]
    fn serialization() {
        let light = Light::default();
        let serialized = serde_json::to_string_pretty(&light).unwrap();    
        serde_json::from_str::<Light>(&serialized).unwrap();
    }
}
