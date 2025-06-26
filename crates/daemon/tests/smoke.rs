use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tokio::time;
use tempfile::NamedTempFile;

use threadrunner_daemon::frame::{read_frame, write_frame};
use threadrunner_daemon::state::DaemonState;
use threadrunner_core::ipc::{PromptRequest, TokenResponse, PROTOCOL_VERSION};

#[tokio::test]
async fn test_daemon_smoke() -> anyhow::Result<()> {
    // Create a unique socket path using tempfile
    let temp_socket = NamedTempFile::new()?;
    let socket_path = temp_socket.path();
    
    // Spawn daemon in background task with custom socket path
    let daemon_socket_path = socket_path.to_path_buf();
    let daemon_handle = tokio::spawn(async move {
        // Set up daemon similar to run_daemon but with custom socket path
        let _ = std::fs::remove_file(&daemon_socket_path); // cleanup if exists
        
        let listener = UnixListener::bind(&daemon_socket_path)?;
        let state = Arc::new(Mutex::new(DaemonState::default()));
        
        // Accept one connection for the test
        let (stream, _) = listener.accept().await?;
        handle_client_test(stream, state).await
    });
    
    // Give daemon time to start
    time::sleep(Duration::from_millis(100)).await;
    
    // Connect to daemon
    let mut client_stream = UnixStream::connect(socket_path).await?;
    
    // Send prompt request
    let request = PromptRequest {
        v: PROTOCOL_VERSION,
        prompt: "lorem ipsum".to_string(),
        stream: true,
    };
    let request_json = serde_json::to_vec(&request)?;
    write_frame(&mut client_stream, &request_json).await?;
    
    // Read framed responses until eos
    let mut tokens = Vec::new();
    loop {
        let response_data = read_frame(&mut client_stream).await?;
        let response: TokenResponse = serde_json::from_slice(&response_data)?;
        
        if let Some(token) = response.token {
            tokens.push(token);
        }
        
        if response.eos {
            break;
        }
    }
    
    // Assert at least one token equals "lorem"
    assert!(
        tokens.iter().any(|token| token == "lorem"),
        "Expected at least one token to equal 'lorem', got tokens: {:?}",
        tokens
    );
    
    // Wait for daemon task to complete
    daemon_handle.await??;
    
    Ok(())
}

// Simplified version of handle_client for testing
async fn handle_client_test(mut stream: UnixStream, state: Arc<Mutex<DaemonState>>) -> anyhow::Result<()> {
    use threadrunner_core::model::{DummyBackend, ModelBackend};
    
    // Read request
    let frame_data = read_frame(&mut stream).await?;
    let request: PromptRequest = serde_json::from_slice(&frame_data)?;
    
    // Lock state and load model if needed
    let mut state_guard = state.lock().await;
    if state_guard.model.is_none() {
        let backend = DummyBackend::load(Path::new("/dev/null"))?;
        state_guard.model = Some(backend);
    }
    
    // Initialize prompt
    let model = state_guard.model.as_mut().unwrap();
    model.prompt(&request.prompt)?;
    drop(state_guard);
    
    // Stream tokens
    loop {
        let mut state_guard = state.lock().await;
        let model = state_guard.model.as_mut().unwrap();
        let tok = model.next_token()?;
        
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