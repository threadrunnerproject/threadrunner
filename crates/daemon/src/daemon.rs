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
use threadrunner_core::model::{BackendKind, load_backend};

/// Get the backend kind from environment variable or use default
fn get_backend_kind() -> anyhow::Result<BackendKind> {
    let backend_str = std::env::var("THREADRUNNER_BACKEND")
        .unwrap_or_else(|_| default_backend().to_string());
    
    parse_backend_env(&backend_str)
}

/// Parse backend string from environment variable
fn parse_backend_env(backend: &str) -> anyhow::Result<BackendKind> {
    match backend.to_lowercase().as_str() {
        #[cfg(feature = "dummy")]
        "dummy" => Ok(BackendKind::Dummy),
        
        #[cfg(feature = "llama")]
        "llama" => Ok(BackendKind::Llama),
        
        _ => {
            let available_backends = available_backends();
            anyhow::bail!(
                "Unknown backend '{}' in THREADRUNNER_BACKEND. Available backends: {}", 
                backend, 
                available_backends.join(", ")
            )
        }
    }
}

/// Returns the default backend based on compiled features
fn default_backend() -> &'static str {
    #[cfg(feature = "llama")]
    return "llama";
    
    #[cfg(all(feature = "dummy", not(feature = "llama")))]
    return "dummy";
    
    #[cfg(not(any(feature = "dummy", feature = "llama")))]
    compile_error!("At least one backend feature must be enabled");
}

/// Get list of available backends based on compiled features
fn available_backends() -> Vec<&'static str> {
    let mut backends = Vec::new();
    
    #[cfg(feature = "dummy")]
    backends.push("dummy");
    
    #[cfg(feature = "llama")]
    backends.push("llama");
    
    backends
}

/// Get the appropriate model path for the given backend kind
fn get_model_path(backend_kind: BackendKind) -> anyhow::Result<std::path::PathBuf> {
    match backend_kind {
        #[cfg(feature = "dummy")]
        BackendKind::Dummy => {
            // Dummy backend doesn't need a real model file
            Ok(std::path::PathBuf::from("/dev/null"))
        }
        
        #[cfg(feature = "llama")]
        BackendKind::Llama => {
            // Use the default model path for Llama backend or environment override
            if let Ok(model_path) = std::env::var("THREADRUNNER_MODEL_PATH") {
                Ok(std::path::PathBuf::from(model_path))
            } else {
                crate::config::default_model_path()
            }
        }
        
        #[cfg(not(feature = "dummy"))]
        BackendKind::Dummy => {
            anyhow::bail!("Dummy backend not available - not compiled with dummy feature")
        }
        
        #[cfg(not(feature = "llama"))]
        BackendKind::Llama => {
            anyhow::bail!("Llama backend not available - not compiled with llama feature")
        }
    }
}

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
                    if let Some(mut model) = state_guard.model.take() {
                        // Use the BoxedModelBackend's unload method
                        if let Err(e) = model.unload() {
                            eprintln!("Error unloading idle model: {}", e);
                        } else {
                            eprintln!("Unloaded idle model");
                        }
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
        let backend_kind = get_backend_kind()?;
        let model_path = get_model_path(backend_kind)?;
        
        let backend_name = match backend_kind {
            #[cfg(feature = "dummy")]
            BackendKind::Dummy => "dummy",
            #[cfg(feature = "llama")]
            BackendKind::Llama => "llama",
            #[cfg(not(feature = "dummy"))]
            BackendKind::Dummy => unreachable!("Dummy backend not compiled"),
            #[cfg(not(feature = "llama"))]
            BackendKind::Llama => unreachable!("Llama backend not compiled"),
        };
        
        eprintln!("Loading {} backend with model: {}", backend_name, model_path.display());
        
        state_guard.model = Some(load_backend(backend_kind, &model_path)?);
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