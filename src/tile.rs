use bytemuck::Pod;
use bytemuck::Zeroable;
use num_derive::FromPrimitive;
use num_derive::ToPrimitive;
use num_traits::FromPrimitive;
use serde::Deserialize;
use serde::Serialize;

use crate::color::Color;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct TileData {
    atlas_position: [i32; 2], // 32 * 2
    color: Color,
    detail: Color,
}

impl TileData {
    fn new(atlas_position: [i32; 2], color: Color, detail: Option<Color>) -> Self {
        Self {
            atlas_position,
            color,
            detail: detail.unwrap_or_default(),
            ..Default::default()
        }
    }
}

#[repr(C, align(8))]
#[derive(Copy, Clone, Debug, Deserialize, FromPrimitive, Serialize, ToPrimitive)]
pub enum Tile {
    Void = 0,
    Player,
    Solid = 128,
    Stone,
    Brick,
}

impl Tile {
    pub fn render_data(self) -> TileData {
        match self {
            Tile::Void => TileData::default(),
            Tile::Solid => TileData::new([1, 0], Color::new(255, 255, 255), None),
            Tile::Stone => TileData::new([2, 0], Color::new(156, 156, 162), None),
            Tile::Brick => TileData::new([3, 0], Color::new(120, 8, 2), None),
            Tile::Player => TileData::new([0, 1], Color::new(200, 0, 0), Some(Color::new(100, 70, 8))),
        }
    }

    pub fn render_atlas() -> [TileData; 256] {
        let mut atlas = vec![];
        for i in 0..=255 {
            let tile: Tile = FromPrimitive::from_u8(i).unwrap_or_default();
            atlas.push(tile.render_data());
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
        Tile::render_atlas();
    }
}
