use winit::dpi::PhysicalSize;

#[derive(Copy, Clone)]
pub struct Size {
    width: u32,
    height: u32,
}

impl Size {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }
}

impl Into<PhysicalSize<u32>> for Size {
    fn into(self) -> PhysicalSize<u32> {
        PhysicalSize::new(self.width, self.height)
    }
}
