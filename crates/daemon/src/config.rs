use std::fs;
use std::path::PathBuf;

pub const SOCKET_PATH: &str = "/tmp/threadrunner.sock";
pub const IDLE_TIMEOUT_SECS: u64 = 300;

/// Returns the default model path for GGUF models
pub fn default_model_path() -> anyhow::Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    
    let model_path = home_dir
        .join(".threadrunner")
        .join("models")
        .join("llama2-7b.Q4_K_M.gguf");
    
    Ok(model_path)
}

/// Removes the socket file if it exists
pub fn cleanup_socket() -> std::io::Result<()> {
    match fs::remove_file(SOCKET_PATH) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
} 