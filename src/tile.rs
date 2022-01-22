use bytemuck::Pod;
use bytemuck::Zeroable;
use num_derive::FromPrimitive;
use num_derive::ToPrimitive;
use num_traits::FromPrimitive;
use serde::Deserialize;
use serde::Serialize;

pub const TILE_SIZE: u32 = 16;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct TileData {
    atlas_position: [u32; 2], // 32 * 2
    color: [u8; 4],
    detail: [u8; 4],
}

impl TileData {
    fn new(atlas_position: [u32; 2], color: [u8; 4], detail: Option<[u8; 4]>) -> Self {
        Self {
            atlas_position,
            color,
            detail: detail.unwrap_or(color),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Deserialize, FromPrimitive, Serialize, ToPrimitive)]
pub enum Tile {
    Void = 0,
    Wall,
    Planks,
}

impl Tile {
    pub fn data(self) -> TileData {
        match self {
            Tile::Wall => TileData::new([0, 1], [255, 255, 255, 255], Some([0, 0, 255, 255])),
            Tile::Planks => TileData::new([2, 1], [255, 255, 255, 255], Some([220, 220, 220, 255])),
            _ => TileData::default(),
        }
    }

    pub fn tiles() -> [TileData; 256] {
        let mut atlas = vec![];
        for i in 0..=255 {
            let tile: Tile = FromPrimitive::from_u8(i).unwrap_or_default();
            atlas.push(tile.data());
        }

        atlas.try_into().unwrap()
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Void
    }
}

#[cfg(test)]
mod tests {
    use super::Tile;

    #[test]
    fn serialization() {
        let tile = Tile::Void;
        let serialized = serde_json::to_string(&tile).unwrap();
        serde_json::from_str::<Tile>(&serialized).unwrap();
    }

    #[test]
    fn atlas_creation() {
        Tile::tiles();
    }
}
