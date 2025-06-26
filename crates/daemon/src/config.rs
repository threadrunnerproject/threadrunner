use std::fs;

pub const SOCKET_PATH: &str = "/tmp/threadrunner.sock";
pub const IDLE_TIMEOUT_SECS: u64 = 300;

/// Removes the socket file if it exists
pub fn cleanup_socket() -> std::io::Result<()> {
    match fs::remove_file(SOCKET_PATH) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
} 