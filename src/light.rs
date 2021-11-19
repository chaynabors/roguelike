use serde::Deserialize;
use serde::Serialize;

use crate::color::Color;
use crate::vector2::Vector2;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Light {
    position: Vector2,
    color: Color,
    luminosity: f32,
}

#[cfg(test)]
mod tests {
    use crate::color::Color;
    use crate::light::Light;
    use crate::vector2::Vector2;

    #[test]
    fn it_works() {
        let vector2 = Vector2{ x: 200., y: 200. };
        let color = Color{ r: 255, g: 255, b: 255 };
        let luminosity = 600.;

        let light = Light{ position: vector2, color: color, luminosity: luminosity };

        let serialized = serde_json::to_string_pretty(&light).unwrap();
        println!("serialized = {}", serialized);
    
        let deserialized: Light = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
    }
}
