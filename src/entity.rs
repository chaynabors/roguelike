use bytemuck::Zeroable;

#[repr(C, align(256))]
#[derive(Clone, Copy, Debug, Zeroable)]
pub struct Entity {
    position: [f32; 2],
    atlas_position: [u32; 2],
    size: [u32; 2],
    color: u32,
    detail: u32,
}

impl Entity {
    pub fn new(
        position: [f32; 2],
        atlas_position: [u32; 2],
        size: [u32; 2],
        color: u32,
        detail: Option<u32>,
    ) -> Self {
        Entity {
            position,
            atlas_position,
            size,
            color,
            detail: detail.unwrap_or(color),
        }
    }
}
