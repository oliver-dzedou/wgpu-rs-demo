use wgpu::PresentMode;

/// Available VSync states
///
/// [VSync::On] maps to [wgpu::PresentMode::AutoVsync]
///
/// [VSync::Off] maps to [wgpu::PresentMode::AutoNoVsync]
///
/// Be aware that some platforms do not support no vsync. In that case, vsync will be turned on even when setting [VSync::Off]
#[derive(Copy, Clone)]
pub enum VSync {
    On,
    Off,
}

impl Into<PresentMode> for VSync {
    fn into(self) -> PresentMode {
        match self {
            Self::On => return PresentMode::AutoVsync,
            Self::Off => return PresentMode::AutoNoVsync,
        };
    }
}
