use serde::Deserialize;
use serde::Serialize;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[cfg(test)]
mod tests {
    use crate::color::Color;

    #[test]
    fn it_works() {
        let color = Color{ r: 255, g: 255, b: 255 };

        let serialized = serde_json::to_string_pretty(&color).unwrap();
        println!("serialized = {}", serialized);
    
        let deserialized: Color = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
    }
}