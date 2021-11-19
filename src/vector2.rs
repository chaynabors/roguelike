use serde::Deserialize;
use serde::Serialize;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[cfg(test)]
mod tests {
    use crate::vector2::Vector2;

    #[test]
    fn it_works() {
        let vector2 = Vector2{ x: 200., y: 200. };

        let serialized = serde_json::to_string_pretty(&vector2).unwrap();
        println!("serialized = {}", serialized);
    
        let deserialized: Vector2 = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
    }
}