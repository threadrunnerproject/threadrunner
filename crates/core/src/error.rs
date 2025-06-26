use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("model load failed: {0}")]
    ModelLoad(#[from] anyhow::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("socket protocol error: {0}")]
    Protocol(String),

    #[error("timeout")]
    Timeout,

    #[error("unknown")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, Error>; 