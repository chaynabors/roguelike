use serde::Deserialize;
use serde::Serialize;

use crate::chunk::Chunk;
use crate::entity::Entity;
use crate::light::Light;

#[derive(Deserialize, Serialize)]
pub struct World {
    pub name: String,
    pub seed: u32,

    #[serde(skip)]
    pub chunks: Vec<Chunk>,
    #[serde(skip)]
    pub entities: Vec<Entity>,
    #[serde(skip)]
    pub lights: Vec<Light>,
}
