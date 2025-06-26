use std::time::Instant;
use threadrunner_core::model::BoxedModelBackend;

pub struct DaemonState {
    pub model: Option<BoxedModelBackend>,
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