use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tokio::time;

use crate::config::{self, SOCKET_PATH, IDLE_TIMEOUT_SECS};
use crate::frame::{read_frame, write_frame};
use crate::state::DaemonState;
use threadrunner_core::ipc::{PromptRequest, TokenResponse};
use threadrunner_core::model::{DummyBackend, ModelBackend};

pub async fn run_daemon() -> anyhow::Result<()> {
    // Clean up any existing socket file
    config::cleanup_socket()?;
    
    // Bind to the Unix socket
    let listener = UnixListener::bind(SOCKET_PATH)?;
    
    // Create shared state wrapped in Arc<Mutex<...>>
    let state = Arc::new(Mutex::new(DaemonState::default()));
    
    // Spawn idle timer task
    let idle_state = state.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            
            let mut state_guard = idle_state.lock().await;
            if let Some(ref mut model) = state_guard.model {
                let elapsed = state_guard.last_activity.elapsed();
                if elapsed.as_secs() > IDLE_TIMEOUT_SECS {
                    // Model is loaded and has been idle too long, unload it
                    if let Err(e) = state_guard.model.take().unwrap().unload() {
                        eprintln!("Error unloading idle model: {}", e);
                    }
                }
            }
        }
    });
    
    // Accept connections and handle them
    loop {
        let (stream, _) = listener.accept().await?;
        let client_state = state.clone();
        
        tokio::spawn(async move {
            if let Err(e) = handle_client(stream, client_state).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
}

async fn handle_client(mut stream: UnixStream, state: Arc<Mutex<DaemonState>>) -> anyhow::Result<()> {
    // Read a frame and parse into PromptRequest
    let frame_data = read_frame(&mut stream).await?;
    let request: PromptRequest = serde_json::from_slice(&frame_data)?;
    
    // Lock state
    let mut state_guard = state.lock().await;
    
    // If no model is loaded, load it
    if state_guard.model.is_none() {
        let backend = DummyBackend::load(Path::new("/dev/null"))?;
        state_guard.model = Some(backend);
    }
    
    // Call model.prompt() and then drop the lock
    let model = state_guard.model.as_mut().unwrap();
    model.prompt(&request.prompt)?;
    drop(state_guard);
    
    // Loop to stream tokens
    loop {
        // Acquire lock and get next token
        let mut state_guard = state.lock().await;
        let model = state_guard.model.as_mut().unwrap();
        let tok = model.next_token()?;
        
        // Update last activity
        state_guard.last_activity = Instant::now();
        
        // Build token response
        let eos = tok.is_none();
        let response = TokenResponse {
            token: tok,
            eos,
        };
        
        // Drop lock before writing
        drop(state_guard);
        
        // Write framed JSON response
        let response_json = serde_json::to_vec(&response)?;
        write_frame(&mut stream, &response_json).await?;
        
        // Break when end-of-stream
        if eos {
            break;
        }
    }
    
    Ok(())
} 