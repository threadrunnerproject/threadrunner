use clap::Parser;
use threadrunner_core::model::BackendKind;
use threadrunner_core::error::{Error, Result};

mod config;
mod client;
mod frame;

#[derive(Debug)]
enum ExitCode {
    Ok = 0,
    Unknown = 1,
    Connection = 2,
    Model = 3,
    Timeout = 4,
}

#[derive(Parser)]
#[command(name = "threadrunner")]
#[command(about = "A thread-based task runner")]
struct Cli {
    /// The prompt to execute
    prompt: Vec<String>,
    
    /// Backend to use for inference
    #[arg(long, default_value = default_backend())]
    backend: String,
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

/// Convert string backend name to BackendKind
fn parse_backend(backend: &str) -> Result<BackendKind> {
    match backend {
        #[cfg(feature = "dummy")]
        "dummy" => Ok(BackendKind::Dummy),
        
        #[cfg(feature = "llama")]
        "llama" => Ok(BackendKind::Llama),
        
        _ => {
            let _available_backends = available_backends();
            Err(Error::Unknown)
        }
    }
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting threadrunner CLI");
    let cli = Cli::parse();
    
    // Join the prompt vector with spaces into a single string
    let prompt = cli.prompt.join(" ");
    tracing::debug!("Processed prompt: {}", prompt);
    
    // Parse and validate backend (for future use)
    let _backend_kind = match parse_backend(&cli.backend) {
        Ok(kind) => kind,
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(ExitCode::Unknown as i32);
        }
    };
    
    match run(prompt).await {
        Ok(_) => {
            std::process::exit(ExitCode::Ok as i32);
        }
        Err(Error::Io(ref io_err)) => {
            eprintln!("Connection error: {:?}", io_err);
            std::process::exit(ExitCode::Connection as i32);
        }
        Err(Error::ModelLoad(_)) => {
            std::process::exit(ExitCode::Model as i32);
        }
        Err(Error::Timeout) => {
            std::process::exit(ExitCode::Timeout as i32);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(ExitCode::Unknown as i32);
        }
    }
}

async fn run(prompt_string: String) -> Result<()> {
    tracing::debug!("Connecting to daemon or spawning if needed");
    let mut stream = client::connect_or_spawn().await?;
    tracing::info!("Successfully connected to daemon");
    
    tracing::debug!("Sending prompt to daemon");
    client::send_prompt(&mut stream, &prompt_string).await?;
    tracing::info!("Finished streaming response");
    
    println!(); // Print newline so shell prompt isn't glued to last token
    Ok(())
} 