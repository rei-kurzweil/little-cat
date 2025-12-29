pub mod ecs;
pub mod graphics;
pub mod networking;
pub mod user_input;
pub mod windowing;
pub mod xr;
pub mod universe;

pub use windowing::Windowing;
pub use universe::Universe;

/// Engine-level error type placeholder.
#[derive(Debug)]
pub enum EngineError {
    NotImplemented,
}

pub type EngineResult<T> = Result<T, EngineError>;
