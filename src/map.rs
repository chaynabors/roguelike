use num_traits::ToPrimitive;
use serde::Deserialize;
use serde::Serialize;

use crate::light::Light;
use crate::tile::Tile;

#[repr(C)]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Map {
    layout: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub lights: Vec<Light>,
}

impl Map {
    pub fn new<const COUNT: usize>(layout: Vec<[Tile; COUNT]>, lights: Vec<Light>) -> Self {
        let mut layout = layout.into_iter().flatten().map(|tile| tile.to_u8().unwrap_or_default()).collect::<Vec<_>>();
        while layout.len() % 4 != 0 {
            layout.push(0);
        }

        let width = COUNT as u32;
        let height = layout.len() as u32;

        Self {
            layout,
            width,
            height,
            lights,
        }
    }

    pub fn layout(&self) -> &[u8] {
        &self.layout
    }
}

#[cfg(test)]
mod tests {
    use super::Map;

    #[test]
    fn serialization() {
        let map = Map::default();
        let serialized = serde_json::to_string(&map).unwrap();
        serde_json::from_str::<Map>(&serialized).unwrap();
    }
}
