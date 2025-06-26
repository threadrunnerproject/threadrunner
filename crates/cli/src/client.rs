use std::io::{self, Write};
use std::io::ErrorKind;
use tokio::net::UnixStream;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{sleep, Duration, Instant};

use crate::config::{daemon_exe, socket_path};
use crate::frame::{read_frame, write_frame};
use threadrunner_core::ipc::{PromptRequest, TokenResponse, ErrorResponse, PROTOCOL_VERSION};
use threadrunner_core::error::{Error, Result};
use anyhow;

/// Connects to the daemon socket, spawning the daemon if necessary
pub async fn connect_or_spawn() -> Result<UnixStream> {
    let socket_path = socket_path().map_err(|e| Error::Io(e))?;
    
    tracing::debug!("Attempting to connect to daemon at: {}", socket_path.display());
    // First attempt to connect
    match UnixStream::connect(&socket_path).await {
        Ok(stream) => {
            tracing::info!("Successfully connected to existing daemon");
            return Ok(stream);
        },
        Err(e) => {
            tracing::debug!("Initial connection failed: {}", e);
            // Only spawn daemon if connection failed due to NotFound or ConnectionRefused
            match e.kind() {
                ErrorKind::NotFound | ErrorKind::ConnectionRefused => {
                    tracing::info!("Daemon not running, attempting to spawn");
                    // Spawn the daemon
                    spawn_daemon().await?;
                }
                _ => {
                    tracing::error!("Connection failed with unexpected error: {}", e);
                    return Err(Error::Io(e));
                }
            }
        }
    }
    
    // Wait up to 5 seconds for daemon to start, retrying connection
    let timeout = Duration::from_secs(5);
    let start_time = Instant::now();
    
    tracing::debug!("Waiting for daemon to start, timeout: {}s", timeout.as_secs());
    loop {
        if start_time.elapsed() >= timeout {
            tracing::error!("Timeout waiting for daemon to start after {} seconds", timeout.as_secs());
            return Err(Error::Timeout);
        }
        
        // Wait a bit before retrying
        sleep(Duration::from_millis(100)).await;
        
        // Try to connect again
        match UnixStream::connect(&socket_path).await {
            Ok(stream) => {
                tracing::info!("Successfully connected to newly spawned daemon");
                return Ok(stream);
            },
            Err(e) => {
                tracing::debug!("Connection retry failed: {}", e);
                // Continue retrying on connection errors
                match e.kind() {
                    ErrorKind::NotFound | ErrorKind::ConnectionRefused => continue,
                    _ => {
                        tracing::error!("Connection retry failed with unexpected error: {}", e);
                        return Err(Error::Io(e));
                    }
                }
            }
        }
    }
}

/// Spawns the daemon process
async fn spawn_daemon() -> Result<()> {
    let daemon_exe_path = daemon_exe().map_err(|e| Error::Io(e))?;
    let socket_path = socket_path().map_err(|e| Error::Io(e))?;
    
    tracing::info!("Spawning daemon process: {:?}", daemon_exe_path);
    let child = Command::new(daemon_exe_path)
        .arg("--socket")
        .arg(socket_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| Error::Io(e))?;
    
    tracing::debug!("Daemon process spawned with PID: {:?}", child.id());
    Ok(())
}

/// Sends a prompt to the daemon and prints streaming tokens to stdout
pub async fn send_prompt(stream: &mut UnixStream, prompt: &str) -> Result<()> {
    // Build PromptRequest with stream: true
    let request = PromptRequest {
        v: PROTOCOL_VERSION,
        prompt: prompt.to_string(),
        stream: true,
    };
    
    tracing::info!("Sending prompt to daemon (length: {} chars)", prompt.len());
    // Serialize via serde_json and write framed bytes
    let request_json = serde_json::to_vec(&request).map_err(|e| Error::Protocol(e.to_string()))?;
    write_frame(stream, &request_json).await.map_err(|e| Error::Io(e))?;
    tracing::debug!("Prompt sent successfully, waiting for response");
    
    let mut token_count = 0;
    // Loop reading frames and try to deserialize as either TokenResponse or ErrorResponse
    loop {
        let response_data = read_frame(stream).await.map_err(|e| Error::Io(e))?;
        
        // First try to parse as ErrorResponse
        if let Ok(error_response) = serde_json::from_slice::<ErrorResponse>(&response_data) {
            tracing::warn!("Received error response from daemon: {} (type: {})", error_response.error, error_response.error_type);
            
            // Convert daemon error to appropriate CLI error based on error_type
            let cli_error = match error_response.error_type.as_str() {
                "ModelLoad" => Error::ModelLoad(anyhow::anyhow!(error_response.error)),
                "Io" => Error::Io(std::io::Error::new(std::io::ErrorKind::Other, error_response.error)),
                "Timeout" => Error::Timeout,
                _ => Error::Protocol(format!("Daemon error: {}", error_response.error)),
            };
            
            return Err(cli_error);
        }
        
        // If not an error response, try to parse as TokenResponse
        let response: TokenResponse = serde_json::from_slice(&response_data)
            .map_err(|e| Error::Protocol(format!("Failed to parse response as token or error: {}", e)))?;
        
        // For each token Some(t) print to stdout without newline, flush after every print
        if let Some(token) = response.token {
            tracing::debug!("Received token: {:?}", token);
            token_count += 1;
            print!("{}", token);
            io::stdout().flush().map_err(|e| Error::Io(e))?;
        }
        
        // Break on eos
        if response.eos {
            tracing::info!("Received end-of-stream, total tokens: {}", token_count);
            break;
        }
    }
    
    Ok(())
} 