use crate::core::surface_provider::SurfaceProvider;
use crate::core::vertex::Vertex;

pub type RenderResult<T> = Result<T, RenderError>;

#[derive(Debug)]
pub struct RenderError;

pub trait Renderer {
    fn init(&mut self, surface: &dyn SurfaceProvider) -> RenderResult<()>;
    fn resize(&mut self, new_size: (u32, u32));
    fn present(&mut self) -> RenderResult<()>;
    fn set_clear_color(&mut self, rgba: [f32; 4]);
    fn submit(&mut self, _vertices: &[Vertex]) {}
}
