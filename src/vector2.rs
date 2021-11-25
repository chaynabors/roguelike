use bytemuck::Pod;
use bytemuck::Zeroable;
use serde::Deserialize;
use serde::Serialize;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Deserialize, Pod, Serialize, Zeroable)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
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
