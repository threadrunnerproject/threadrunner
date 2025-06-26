pub mod model;
pub mod ipc;
pub mod error;
#[cfg(feature = "llama")]
pub mod llama_backend;

pub use model::ModelBackend;
pub use ipc::{PromptRequest, TokenResponse, PROTOCOL_VERSION};
pub use error::{Error, Result}; 