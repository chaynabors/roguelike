use bytemuck::Zeroable;
use serde::Deserialize;
use serde::Serialize;

#[repr(C, align(256))]
#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize, Zeroable)]
pub struct Light {
    position: [f32; 2],
    color: [u8; 3],
    magnitude: u8,
}

impl Light {
    pub fn new(position: [f32; 2], color: [u8; 3], magnitude: u8) -> Self {
        Self { position, color, magnitude }
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
