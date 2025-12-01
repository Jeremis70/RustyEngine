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
}

pub type BackendResult<T> = Result<T, BackendError>;

/// Backend trait abstracts platform windowing and event loop.
pub trait Backend {
    fn create_window(&mut self, config: crate::core::window_config::WindowConfig) -> BackendResult<()>;
    fn run(
        &mut self,
        handler: &mut dyn crate::core::event_handler::EventHandlerApi,
        state: &mut crate::core::engine_state::EngineState,
    ) -> BackendResult<()>;
}
