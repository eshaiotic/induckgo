//! The Guard engine as the request policy and the page-hiding bridge share it.
//!
//! Building the engine from the downloaded lists takes seconds, so it is built on a worker thread while the
//! browser already runs; until it is installed Guard blocks and hides nothing.

use std::sync::OnceLock;

use crate::guard::guard_engine::GuardEngine;

#[derive(Default)]
pub struct SharedGuardEngine {
    engine: OnceLock<GuardEngine>,
}

impl SharedGuardEngine {
    /// Makes `engine` the one in use; a second call is ignored.
    pub fn install(&self, engine: GuardEngine) {
        let _ = self.engine.set(engine);
    }

    #[must_use]
    pub fn get(&self) -> Option<&GuardEngine> {
        self.engine.get()
    }
}
