use serde::Deserialize;
use serde::Serialize;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Material {
    Grass,
    Rubble,
    Pavement,
}
