use serde::Deserialize;
use serde::Serialize;

use crate::light::Light;
use crate::material::Material;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Map {
    layout: Vec<Material>,
    width: u32,
    lights: Vec<Light>,

}

#[cfg(test)]
mod tests {
    use super::Map;

    #[test]
    fn serialization() {
        let map = Map::default();
        let serialized = serde_json::to_string_pretty(&map).unwrap();    
        serde_json::from_str::<Map>(&serialized).unwrap();
    }
}
