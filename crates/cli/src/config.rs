use anyhow::{anyhow, Context, Result};
use directories::BaseDirs;
use std::path::PathBuf;

/// Configuration-related errors
#[derive(Debug)]
pub enum ConfigError {
    HomeDirectoryNotFound,
    CurrentExeNotFound,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::HomeDirectoryNotFound => write!(f, "Failed to determine home directory"),
            ConfigError::CurrentExeNotFound => write!(f, "Failed to determine current executable path"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Path-related errors
#[derive(Debug)]
pub enum PathError {
    InvalidExePath(String),
    DaemonExeResolution,
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathError::InvalidExePath(path) => write!(f, "Invalid executable path: {}", path),
            PathError::DaemonExeResolution => write!(f, "Failed to resolve daemon executable path"),
        }
    }
}

impl std::error::Error for PathError {}

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
        .ok_or_else(|| PathError::InvalidExePath(
            current_exe.display().to_string()
        ))?;
    
    let daemon_exe = parent_dir.join("threadrunner-daemon");
    Ok(daemon_exe)
} 