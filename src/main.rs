use std::io;

use clap::{Parser, Subcommand};
use commands::tmux::tmux;
use config::load_config;
mod commands;
mod config;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    name: Option<String>,

    #[arg(short, long, value_name = "FILE", global = true)]
    config: Option<String>,

    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run tmux sessions
    Tmux,
}

fn main() -> Result<(), io::Error> {
    let cli = Cli::parse();

    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    let config = load_config(cli.config)?;

    match &cli.command {
        Some(Commands::Tmux) => tmux(config),
        None => {}
    }

    Ok(())
}
