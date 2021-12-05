use serde::Deserialize;
use serde::Serialize;
use wgpu::Color;

use crate::light::Light;
use crate::tile::Tile;

pub const CHUNK_SIZE: u32 = 16;
pub const CHUNK_CLEAR_COLOR: Color = Color { r: 0.05, g: 0.05, b: 0.05, a: 1.0 };

pub type ChunkLayout = [[Tile; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Chunk {
    pub layout: ChunkLayout,
    pub lights: Vec<Light>,
}

#[cfg(test)]
mod tests {
    use super::Chunk;

    #[test]
    fn serialization() {
        let chunk = Chunk::default();
        let serialized = serde_json::to_string(&chunk).unwrap();
        serde_json::from_str::<Chunk>(&serialized).unwrap();
    }
}
