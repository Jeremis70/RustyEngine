use crate::backend::surface_provider::SurfaceProvider;
use crate::backend::window::WindowConfig;
use crate::core::assets::ImageId;
use crate::render::{SpriteDrawData, Vertex};
use thiserror::Error;

pub type RenderResult<T> = Result<T, RenderError>;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to initialize renderer: {0}")]
    InitFailed(String),

    #[error("Shader compilation failed:\n{0}")]
    ShaderCompilation(String),

    #[error("GPU memory exhausted")]
    OutOfMemory,

    #[error("Device lost (GPU reset?)")]
    DeviceLost,

    #[error("Invalid texture format: {0}")]
    InvalidTexture(String),

    #[error("Rendering operation failed: {0}")]
    RenderFailed(String),

    #[error("Window surface error: {0}")]
    SurfaceError(String),

    #[error("Pipeline setup failed: {0}")]
    PipelineSetup(String),
}

pub trait Renderer {
    fn init(
        &mut self,
        surface: &dyn SurfaceProvider,
        config: Option<&WindowConfig>,
    ) -> RenderResult<()>;
    fn resize(&mut self, new_size: (u32, u32));
    fn present(&mut self) -> RenderResult<()>;
    fn set_clear_color(&mut self, rgba: [f32; 4]);
    fn submit(&mut self, _vertices: &[Vertex]) {}

    /// Upload an RGBA8 image as a GPU texture associated with the given id.
    fn upload_image(
        &mut self,
        _id: ImageId,
        _width: u32,
        _height: u32,
        _data: &[u8],
    ) -> RenderResult<()> {
        Ok(())
    }

    /// Draw a list of sprites for the current frame.
    fn draw_sprites(&mut self, _sprites: &[SpriteDrawData], _viewport_size: (u32, u32)) {}
}
