use serde::Deserialize;
use serde::Serialize;

#[derive(Copy, Clone, Debug, Default, Deserialize, Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn serialization() {
        let color = Color::default();
        let serialized = serde_json::to_string(&color).unwrap();    
        serde_json::from_str::<Color>(&serialized).unwrap();
    }
}
