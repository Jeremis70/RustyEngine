use thiserror::Error;

/// Errors that can occur in a backend implementation.
#[derive(Debug, Error)]
pub enum BackendError {
    #[error("event loop already consumed")]
    EventLoopConsumed,

    #[error("window creation failed: {0}")]
    WindowCreationFailed(String),

    #[error("platform error: {0}")]
    PlatformError(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("renderer setup failed: {0}")]
    RendererSetupFailed(String),
    #[error("renderer init failed")]
    RendererInit,
}

pub type BackendResult<T> = Result<T, BackendError>;

/// Backend trait abstracts platform windowing and event loop.
pub trait WindowBackend {
    fn create_window(
        &mut self,
        config: crate::backend::window::WindowConfig,
    ) -> BackendResult<()>;
    fn run(
        &mut self,
        handler: &mut dyn crate::core::events::EventHandlerApi,
    ) -> BackendResult<()>;

    /// Returns a surface provider if the window has been created.
    fn surface_provider(&self) -> Option<&dyn crate::backend::surface_provider::SurfaceProvider>;
}
