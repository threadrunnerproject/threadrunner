use anyhow::{anyhow, Result};
use std::io::{self, Write};
use std::io::ErrorKind;
use tokio::net::UnixStream;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{sleep, Duration, Instant};

use crate::config::{daemon_exe, socket_path};
use crate::frame::{read_frame, write_frame};
use threadrunner_core::ipc::{PromptRequest, TokenResponse, PROTOCOL_VERSION};

/// Connects to the daemon socket, spawning the daemon if necessary
pub async fn connect_or_spawn() -> Result<UnixStream> {
    let socket_path = socket_path()?;
    
    // First attempt to connect
    match UnixStream::connect(&socket_path).await {
        Ok(stream) => return Ok(stream),
        Err(e) => {
            // Only spawn daemon if connection failed due to NotFound or ConnectionRefused
            match e.kind() {
                ErrorKind::NotFound | ErrorKind::ConnectionRefused => {
                    // Spawn the daemon
                    spawn_daemon().await?;
                }
                _ => return Err(anyhow!("Failed to connect to daemon: {}", e)),
            }
        }
    }
    
    // Wait up to 5 seconds for daemon to start, retrying connection
    let timeout = Duration::from_secs(5);
    let start_time = Instant::now();
    
    loop {
        if start_time.elapsed() >= timeout {
            return Err(anyhow!(
                "Timeout waiting for daemon to start after {} seconds", 
                timeout.as_secs()
            ));
        }
        
        // Wait a bit before retrying
        sleep(Duration::from_millis(100)).await;
        
        // Try to connect again
        match UnixStream::connect(&socket_path).await {
            Ok(stream) => return Ok(stream),
            Err(e) => {
                // Continue retrying on connection errors
                match e.kind() {
                    ErrorKind::NotFound | ErrorKind::ConnectionRefused => continue,
                    _ => return Err(anyhow!("Failed to connect to daemon: {}", e)),
                }
            }
        }
    }
}

/// Spawns the daemon process
async fn spawn_daemon() -> Result<()> {
    let daemon_exe_path = daemon_exe()?;
    let socket_path = socket_path()?;
    
    Command::new(daemon_exe_path)
        .arg("--socket")
        .arg(socket_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    
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
    
    // Serialize via serde_json and write framed bytes
    let request_json = serde_json::to_vec(&request)?;
    write_frame(stream, &request_json).await?;
    
    // Loop reading frames and deserialize TokenResponse
    loop {
        let response_data = read_frame(stream).await?;
        let response: TokenResponse = serde_json::from_slice(&response_data)?;
        
        // For each token Some(t) print to stdout without newline, flush after every print
        if let Some(token) = response.token {
            print!("{}", token);
            io::stdout().flush()?;
        }
        
        // Break on eos
        if response.eos {
            break;
        }
    }
    
    Ok(())
} 