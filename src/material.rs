use serde::Deserialize;
use serde::Serialize;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Material {
    _None
}

#[cfg(test)]
mod tests {
    use super::Material;

    #[test]
    fn serialization() {
        let material = Material::_None;
        let serialized = serde_json::to_string_pretty(&material).unwrap();    
        serde_json::from_str::<Material>(&serialized).unwrap();
    }
}
