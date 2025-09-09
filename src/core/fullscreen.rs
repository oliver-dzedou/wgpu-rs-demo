#[derive(Clone, Copy)]
/// Available full screen modes
pub enum Fullscreen {
    Borderless,
    No,
}

impl Into<Option<winit::window::Fullscreen>> for Fullscreen {
    fn into(self) -> Option<winit::window::Fullscreen> {
        match self {
            Fullscreen::Borderless => Some(winit::window::Fullscreen::Borderless(None)),
            Fullscreen::No => None,
        }
    }
}
