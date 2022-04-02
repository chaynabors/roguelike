use winit::dpi::PhysicalSize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl From<PhysicalSize<u32>> for Resolution {
    fn from(from: PhysicalSize<u32>) -> Self {
        Self { width: from.width, height: from.height }
    }
}

impl Into<PhysicalSize<u32>> for Resolution {
    fn into(self) -> PhysicalSize<u32> {
        PhysicalSize { width: self.width, height: self.height }
    }
}
