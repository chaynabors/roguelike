use bytemuck::Zeroable;
use serde::Deserialize;
use serde::Serialize;

use crate::color::Color;

#[repr(C, align(256))]
#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize, Zeroable)]
pub struct Light {
    color: Color,
    magnitude: f32,
}

impl Light {
    pub fn new(color: Color, magnitude: f32) -> Self {
        Self { color, magnitude }
    }
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
