use serde::Deserialize;
use serde::Serialize;

use crate::light::Light;
use crate::material::Material;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Map {
    layout: Vec<Material>,
    lights: Vec<Light>,
}
