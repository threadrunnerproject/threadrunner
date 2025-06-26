#![allow(clippy::unused_async)]

mod config;
mod state;
mod frame;
mod daemon;

use daemon::run_daemon;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run_daemon().await
} 

