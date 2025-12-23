use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

/// Backend-agnostic interface to provide window/display handles and size
/// for renderers like wgpu without depending on winit.
pub trait SurfaceProvider: HasWindowHandle + HasDisplayHandle {
    fn size(&self) -> (u32, u32);
}
