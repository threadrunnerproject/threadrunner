pub mod model;
pub mod ipc;
pub use model::ModelBackend;
pub use ipc::{PromptRequest, TokenResponse, PROTOCOL_VERSION}; 