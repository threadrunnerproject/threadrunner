use std::time::Instant;

pub struct DaemonState {
    pub model: Option<threadrunner_core::model::DummyBackend>,
    pub last_activity: Instant,
}

impl Default for DaemonState {
    fn default() -> Self {
        Self {
            model: None,
            last_activity: Instant::now(),
        }
    }
} 