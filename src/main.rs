mod adb;
mod cli;
mod input;
mod logcat;
mod mcp;
mod screen;
mod state;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("abridge=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Command::Screen(args) => screen::run(args).await,
        Command::Log(args) => logcat::run(args).await,
        Command::Input(args) => input::run(args).await,
        Command::State(args) => state::run(args).await,
        Command::Crash(args) => state::crash(args).await,
        Command::Devices(args) => adb::connection::run(args).await,
        Command::Serve => mcp::serve().await,
    }
}
