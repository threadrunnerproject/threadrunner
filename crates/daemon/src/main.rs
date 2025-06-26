#![allow(clippy::unused_async)]

mod config;
mod state;
mod frame;
mod daemon;

use daemon::run_daemon;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let file_appender = tracing_appender::rolling::daily(
        dirs::cache_dir().unwrap(),
        "threadrunner-daemon.log",
    );
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let result = run_daemon().await;
    
    // Keep _guard alive to flush file
    drop(_guard);
    
    result
} 

