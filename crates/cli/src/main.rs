use anyhow::Result;
use clap::Parser;
use threadrunner_core::model::BackendKind;

mod config;
mod client;
mod frame;

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
            let available_backends = available_backends();
            anyhow::bail!(
                "Unknown backend '{}'. Available backends: {}", 
                backend, 
                available_backends.join(", ")
            )
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
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Join the prompt vector with spaces into a single string
    let prompt = cli.prompt.join(" ");
    
    // Parse and validate backend (for future use)
    let _backend_kind = parse_backend(&cli.backend)?;
    
    run(prompt).await
}

async fn run(prompt_string: String) -> Result<()> {
    let mut stream = client::connect_or_spawn().await?;
    client::send_prompt(&mut stream, &prompt_string).await?;
    println!(); // Print newline so shell prompt isn't glued to last token
    Ok(())
} 