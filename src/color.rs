use bytemuck::Pod;
use bytemuck::Zeroable;
use serde::Deserialize;
use serde::Serialize;

#[repr(C, align(4))]
#[derive(Copy, Clone, Debug, Default, Deserialize, Pod, Serialize, Zeroable)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    _pad: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, _pad: 255 }
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn serialization() {
        let color = Color::default();
        let serialized = serde_json::to_string(&color).unwrap();    
        serde_json::from_str::<Color>(&serialized).unwrap();
    }
}
