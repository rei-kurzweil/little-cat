use crate::engine::EngineResult;

/// Networking placeholder.
///
/// If you want a higher-level protocol later, we can layer it on top of UDP/TCP.
pub struct Networking;

impl Networking {
    pub fn new() -> EngineResult<Self> {
        Ok(Self)
    }
}
