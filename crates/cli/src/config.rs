use anyhow::{Context, Result};
use std::path::PathBuf;

/// Returns the path to the ThreadRunner socket file in the user's home directory
pub fn socket_path() -> Result<PathBuf> {
    // For now, use the same hardcoded path as the daemon
    // TODO: Make this configurable and consistent between CLI and daemon
    Ok(PathBuf::from("/tmp/threadrunner.sock"))
}

/// Returns the path to the threadrunner-daemon executable
/// by resolving it as a sibling to the current executable
pub fn daemon_exe() -> Result<PathBuf> {
    let current_exe = std::env::current_exe()
        .context("Failed to get current executable path")?;
    
    let parent_dir = current_exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid executable path: {}", current_exe.display()))?;
    
    let daemon_exe = parent_dir.join("threadrunner-daemon");
    Ok(daemon_exe)
} 