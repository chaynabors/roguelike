use enum_iterator::IntoEnumIterator;

#[repr(u8)]
#[derive(Copy, Clone, Debug, IntoEnumIterator)]
pub enum Material {
    Void = 0,
    Solid,
    Wall,
    OrderlyTwist,
    UncutTile,
}

impl Material {
    pub fn texture_path(self) -> &'static str {
        match self {
            Material::Void => "void",
            Material::Solid => "solid",
            Material::Wall => "wall",
            Material::OrderlyTwist => "orderly_twist",
            Material::UncutTile => "uncut_tile",
        }
    }
}
