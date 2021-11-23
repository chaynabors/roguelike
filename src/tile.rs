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
pub struct TileRenderData {
    atlas_position: [i32; 2], // 32 * 2
    color: Color,
    detail: Color,
}

impl TileRenderData {
    fn new(atlas_position: [i32; 2], color: Color, detail: Option<Color>) -> Self {
        Self {
            atlas_position,
            color,
            detail: detail.unwrap_or_default(),
            ..Default::default()
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Deserialize, FromPrimitive, Serialize, ToPrimitive)]
pub enum Tile {
    Void = 0, Solid, Stone, Brick,
    Player,
}

impl Tile {
    pub fn render_data(self) -> TileRenderData {
        match self {
            Tile::Void => TileRenderData::default(),
            Tile::Solid => TileRenderData::new([1, 0], Color::new(255, 255, 255), None),
            Tile::Stone => TileRenderData::new([2, 0], Color::new(156, 156, 162), None),
            Tile::Brick => TileRenderData::new([3, 0], Color::new(120, 8, 2), None),
            Tile::Player => TileRenderData::new([0, 1], Color::new(200, 200, 200), Some(Color::new(100, 70, 8))),
        }
    }

    pub fn render_atlas() -> [TileRenderData; 256] {
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
        let material = Tile::Void;
        let serialized = serde_json::to_string(&material).unwrap();
        serde_json::from_str::<Tile>(&serialized).unwrap();
    }

    #[test]
    fn atlas_creation() {
        Tile::render_atlas();
    }
}
