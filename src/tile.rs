use bytemuck::Pod;
use bytemuck::Zeroable;
use num_derive::FromPrimitive;
use num_derive::ToPrimitive;
use num_traits::FromPrimitive;
use serde::Deserialize;
use serde::Serialize;

use crate::material::Material;

pub const TILE_SIZE: u32 = 16;

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Pod, Zeroable)]
pub struct TileData {
    material: u32,
    primary_color: [u8; 4],
    secondary_color: [u8; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Deserialize, FromPrimitive, Serialize, ToPrimitive)]
pub enum Tile {
    Void = 0,
    Wall,
    Planks,
}

impl Tile {
    /// Get the material of a specific tile
    pub fn material(self) -> Material {
        match self {
            Tile::Void => Material::Void,
            Tile::Wall => Material::Wall,
            Tile::Planks => Material::OrderlyTwist,
        }
    }

    /// Get the primary color of a specific tile
    pub fn primary_color(self) -> [u8; 4] {
        match self {
            Tile::Void => [0, 0, 0, 0],
            Tile::Wall => [255, 255, 255, 255],
            Tile::Planks => [255, 255, 255, 255],
        }
    }

    /// Get the secondary color of a specific tile
    pub fn secondary_color(self) -> [u8; 4] {
        match self {
            Tile::Wall => [0, 0, 255, 255],
            Tile::Planks => [220, 220, 220, 255],
            _ => [0, 0, 0, 0],
        }
    }

    pub fn tile_data() -> Vec<TileData> {
        let mut atlas = vec![];
        for i in 0..=255 {
            let tile: Tile = FromPrimitive::from_u8(i).unwrap_or(Tile::Void);
            atlas.push(TileData {
                material: tile.material() as u32,
                primary_color: tile.primary_color(),
                secondary_color: tile.secondary_color(),
            });
        }

        atlas
    }
}
