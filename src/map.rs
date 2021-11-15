use crate::light::Light;
use crate::material::Material;

#[derive(Clone, Debug)]
pub struct Map {
    layout: Vec<Material>,
    lights: Vec<Light>,
}
