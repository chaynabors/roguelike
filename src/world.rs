use serde::Deserialize;
use serde::Serialize;

use crate::chunk::Chunk;

#[derive(Deserialize, Serialize)]
struct World {
    name: String,
    seed: u32,

    #[serde(skip_serializing)]
    loaded_chunks: Vec<Chunk>,
}
