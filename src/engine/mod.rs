pub mod camera;
pub mod ecs;
pub mod graphics;
pub mod networking;
pub mod user_input;
pub mod windowing;
pub mod xr;

/// Engine-level error type placeholder.
#[derive(Debug)]
pub enum EngineError {
    NotImplemented,
}

pub type EngineResult<T> = Result<T, EngineError>;
