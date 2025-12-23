use crate::engine::{EngineError, EngineResult};

/// Minimal winit wrapper placeholder.
///
/// We keep this tiny for now so the project compiles and we can evolve the API later.
pub struct Windowing;

impl Windowing {
    pub fn new() -> EngineResult<Self> {
        // TODO: create winit::event_loop::EventLoop and a Window.
        Ok(Self)
    }

    pub fn run(self) -> EngineResult<()> {
        // TODO: start winit event loop.
        Err(EngineError::NotImplemented)
    }
}
