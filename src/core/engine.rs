use crate::backend::backend::{Backend, BackendError, BackendResult};
use crate::core::engine_state::EngineState;
use crate::core::event_handler::EventHandler;
use crate::core::window_config::WindowConfig;

pub struct Engine {
    pub events: EventHandler,
    pub state: EngineState,
    backend: Box<dyn Backend>,
}

impl Engine {
    pub fn new(backend: Box<dyn Backend>) -> Self {
        Self {
            events: EventHandler::new(),
            state: EngineState::new(),
            backend,
        }
    }

    /// Create a window via the backend. Returns an error if the backend fails.
    pub fn create_window(&mut self, config: WindowConfig) -> BackendResult<()> {
        // Validate window configuration before passing to backend
        config.validate().map_err(BackendError::InvalidConfig)?;
        self.backend.create_window(config)
    }

    /// Run the backend event loop. Returns an error if the backend fails.
    pub fn run(&mut self) -> BackendResult<()> {
        self.backend.run(&mut self.events, &mut self.state)
    }
}
