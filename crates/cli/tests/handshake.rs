use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_cli_daemon_handshake() -> anyhow::Result<()> {
    // Use the daemon's hardcoded socket path for now
    let socket_path = PathBuf::from("/tmp/threadrunner.sock");
    
    // Clean up any existing socket file
    let _ = std::fs::remove_file(&socket_path);
    
    // Build paths to the binaries (assumes they're built in target/debug)
    let daemon_binary = get_binary_path("threadrunner-daemon")?;
    let cli_binary = get_binary_path("threadrunner")?;
    
    // Spawn the daemon process (no arguments needed - it uses hardcoded socket)
    let mut daemon_child = Command::new(&daemon_binary)
        .spawn()?;
    
    // Give the daemon a moment to start up and bind to the socket
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Run the CLI binary with "lorem ipsum" prompt, capturing stdout
    let cli_output = timeout(
        Duration::from_secs(3),
        Command::new(&cli_binary)
            .arg("lorem")
            .arg("ipsum")
            .output()
    ).await??;
    
    // Convert stdout to string for assertion
    let stdout_text = String::from_utf8(cli_output.stdout)?;
    
    // Assert the output contains "lorem" (from the dummy backend)
    assert!(
        stdout_text.contains("lorem"),
        "CLI output should contain 'lorem', got: {:?}",
        stdout_text
    );
    
    // Assert the CLI exited successfully
    assert!(
        cli_output.status.success(),
        "CLI should exit with success status, got: {:?}",
        cli_output.status
    );
    
    // Clean up: terminate the daemon process
    daemon_child.kill().await?;
    daemon_child.wait().await?;
    
    Ok(())
}

/// Helper function to resolve binary paths in the target directory
fn get_binary_path(binary_name: &str) -> anyhow::Result<PathBuf> {
    // Get the current executable path and navigate to the target/debug directory
    let current_exe = std::env::current_exe()?;
    
    // Navigate from target/debug/deps to target/debug
    let target_debug_dir = current_exe
        .parent() // Remove binary name
        .and_then(|p| p.parent()) // Remove "deps"
        .ok_or_else(|| anyhow::anyhow!("Failed to get target/debug directory"))?;
    
    let binary_path = target_debug_dir.join(binary_name);
    
    // Ensure the binary exists
    if !binary_path.exists() {
        return Err(anyhow::anyhow!(
            "Binary {} not found at {}. Make sure to build the project first.",
            binary_name,
            binary_path.display()
        ));
    }
    
    Ok(binary_path)
} 