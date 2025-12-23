use crate::engine::{EngineError, EngineResult};

/// OpenXR session handling placeholder.
pub struct Xr;

impl Xr {
    pub fn new() -> EngineResult<Self> {
        // TODO: create openxr::Instance, system, session.
        Ok(Self)
    }

    pub fn begin_session(&mut self) -> EngineResult<()> {
        Err(EngineError::NotImplemented)
    }
}
