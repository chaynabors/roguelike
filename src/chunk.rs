use wgpu::Color;

pub const CHUNK_SIZE: u32 = 16;
pub const CHUNK_CLEAR_COLOR: Color = Color { r: 0.01, g: 0.01, b: 0.01, a: 0.0 };

pub type Chunk = [[u8; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
