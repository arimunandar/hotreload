use clap::Parser;
use tracing_subscriber::EnvFilter;

mod cli;
mod commands;
mod compiler;
mod config;
mod error;
mod injector;
mod server;
mod watcher;
mod xcode;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = cli::Cli::parse();
    cli.exec()
}
