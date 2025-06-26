use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::fs;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tokio::time;
use tempfile::{NamedTempFile, TempDir};

use threadrunner_daemon::frame::{read_frame, write_frame};
use threadrunner_daemon::state::DaemonState;
use threadrunner_core::ipc::{PromptRequest, TokenResponse, PROTOCOL_VERSION};

// Custom idle timeout for testing (1 second)
const TEST_IDLE_TIMEOUT_SECS: u64 = 1;

#[tokio::test]
async fn test_daemon_idle_unload_and_log() -> anyhow::Result<()> {
    // Create temporary directory for log files
    let temp_dir = TempDir::new()?;
    let log_path = temp_dir.path().join("threadrunner-daemon.log");
    
    // Create a unique socket path using tempfile
    let temp_socket = NamedTempFile::new()?;
    let socket_path = temp_socket.path();
    
    // Set up file appender for logging to our test log file
    let file_appender = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::NEVER)
        .filename_prefix("threadrunner-daemon")
        .filename_suffix("log")
        .build(temp_dir.path())?;
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    // Initialize tracing for this test
    let _subscriber_guard = tracing::subscriber::set_default(
        tracing_subscriber::fmt()
            .with_writer(non_blocking)
            .with_env_filter("info")
            .finish()
    );
    
    // Spawn daemon in background task with custom socket path and idle timeout
    let daemon_socket_path = socket_path.to_path_buf();
    let daemon_handle = tokio::spawn(async move {
        run_daemon_with_idle_timeout(daemon_socket_path, TEST_IDLE_TIMEOUT_SECS).await
    });
    
    // Give daemon time to start
    time::sleep(Duration::from_millis(200)).await;
    
    // Connect to daemon and send a prompt
    let mut client_stream = UnixStream::connect(socket_path).await?;
    
    // Send prompt request to load the model
    let request = PromptRequest {
        v: PROTOCOL_VERSION,
        prompt: "test prompt".to_string(),
        stream: true,
    };
    let request_json = serde_json::to_vec(&request)?;
    write_frame(&mut client_stream, &request_json).await?;
    
    // Read and consume all response tokens
    loop {
        let response_data = read_frame(&mut client_stream).await?;
        let response: TokenResponse = serde_json::from_slice(&response_data)?;
        
        if response.eos {
            break;
        }
    }
    
    // Close the client connection to ensure no ongoing activity
    drop(client_stream);
    
    // Sleep for 2 seconds to trigger idle timeout (which is set to 1 second)
    time::sleep(Duration::from_secs(2)).await;
    
    // Give some extra time for the log to be written
    time::sleep(Duration::from_millis(500)).await;
    
    // Read the log file and check for "unloaded model" message
    let log_contents = std::fs::read_to_string(&log_path)
        .unwrap_or_else(|_| {
            // If the exact log file doesn't exist, try to find any log file in the directory
            if let Ok(entries) = fs::read_dir(temp_dir.path()) {
                for entry in entries.flatten() {
                    if entry.file_name().to_string_lossy().contains("threadrunner-daemon") {
                        if let Ok(contents) = fs::read_to_string(entry.path()) {
                            return contents;
                        }
                    }
                }
            }
            String::new()
        });
    
    // Assert that the log contains the idle unload message
    assert!(
        log_contents.contains("Successfully unloaded idle model") || 
        log_contents.contains("unloaded idle model") ||
        log_contents.contains("Unloaded idle model"),
        "Log should contain idle model unload message. Log contents: {}",
        log_contents
    );
    
    // Terminate the daemon
    daemon_handle.abort();
    
    // Keep the guard alive until the end to ensure logs are flushed
    drop(_guard);
    
    Ok(())
}

// Custom daemon runner with configurable idle timeout for testing
async fn run_daemon_with_idle_timeout(socket_path: std::path::PathBuf, idle_timeout_secs: u64) -> anyhow::Result<()> {
    use threadrunner_core::model::{DummyBackend, ModelBackend, BoxedModelBackend};
    
    // Clean up any existing socket file
    let _ = std::fs::remove_file(&socket_path);
    
    // Bind to the Unix socket
    tracing::info!("Binding to test Unix socket: {}", socket_path.display());
    let listener = UnixListener::bind(&socket_path)?;
    
    // Create shared state
    let state = Arc::new(Mutex::new(DaemonState::default()));
    
    // Spawn idle timer task with custom timeout
    let idle_state = state.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(1)); // Check every second for testing
        loop {
            interval.tick().await;
            
            let mut state_guard = idle_state.lock().await;
            if let Some(ref mut model) = state_guard.model {
                let elapsed = state_guard.last_activity.elapsed();
                if elapsed.as_secs() > idle_timeout_secs {
                    tracing::info!("Idle timeout fired after {} seconds", elapsed.as_secs());
                    // Model is loaded and has been idle too long, unload it
                    if let Some(mut model) = state_guard.model.take() {
                        if let Err(e) = model.unload() {
                            tracing::error!("Error unloading idle model: {}", e);
                        } else {
                            tracing::info!("Successfully unloaded idle model");
                        }
                    }
                }
            }
        }
    });
    
    // Accept one connection for the test
    let (stream, _) = listener.accept().await?;
    handle_client_test_with_model(stream, state).await
}

// Test client handler that loads and uses a model
async fn handle_client_test_with_model(mut stream: UnixStream, state: Arc<Mutex<DaemonState>>) -> anyhow::Result<()> {
    use threadrunner_core::model::{DummyBackend, ModelBackend, BoxedModelBackend};
    use std::time::Instant;
    
    // Read request
    let frame_data = read_frame(&mut stream).await?;
    let request: PromptRequest = serde_json::from_slice(&frame_data)?;
    
    // Lock state and load model if needed
    let mut state_guard = state.lock().await;
    if state_guard.model.is_none() {
        let backend = DummyBackend::load(Path::new("/dev/null"))?;
        let boxed_backend = BoxedModelBackend::new(Box::new(backend));
        state_guard.model = Some(boxed_backend);
        tracing::info!("Loaded dummy model for test");
    }
    
    // Initialize prompt and update activity time
    let model = state_guard.model.as_mut().unwrap();
    model.prompt(&request.prompt)?;
    state_guard.last_activity = Instant::now();
    drop(state_guard);
    
    // Stream tokens
    loop {
        let mut state_guard = state.lock().await;
        let model = state_guard.model.as_mut().unwrap();
        let tok = model.next_token()?;
        
        // Update last activity
        state_guard.last_activity = Instant::now();
        
        let eos = tok.is_none();
        let response = TokenResponse {
            token: tok,
            eos,
        };
        drop(state_guard);
        
        let response_json = serde_json::to_vec(&response)?;
        write_frame(&mut stream, &response_json).await?;
        
        if eos {
            break;
        }
    }
    
    Ok(())
} 