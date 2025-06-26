use anyhow::Result;
use clap::Parser;

mod config;
mod client;
mod frame;

#[derive(Parser)]
#[command(name = "threadrunner")]
#[command(about = "A thread-based task runner")]
struct Cli {
    /// The prompt to execute
    prompt: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Join the prompt vector with spaces into a single string
    let prompt = cli.prompt.join(" ");
    
    run(prompt).await
}

async fn run(prompt_string: String) -> Result<()> {
    let mut stream = client::connect_or_spawn().await?;
    client::send_prompt(&mut stream, &prompt_string).await?;
    println!(); // Print newline so shell prompt isn't glued to last token
    Ok(())
} 