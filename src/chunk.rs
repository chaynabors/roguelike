use serde::Deserialize;
use serde::Serialize;

use crate::light::Light;
use crate::tile::Tile;
use crate::vector2::Vector2;

pub const CHUNK_SIZE: u32 = 16;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Chunk {
    pub layout: [[Tile; CHUNK_SIZE as usize]; CHUNK_SIZE as usize],
    pub lights: Vec<(Light, Vector2)>,
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
