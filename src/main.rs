use std::io;

use clap::{Args, Parser, Subcommand};
use commands::{
    tmux::tmux,
    workspaces::{
        current::{get_current_workspace, CurrentWorkspaceOptions},
        list::{list_workspaces, ListWorkspaceOptions},
    },
};
use config::load_config;
use utils::workspace::{
    JsonPrettyWorkspaceDisplay, JsonWorkspaceDisplay, PrettyWorkspaceDisplay, WorkspaceDisplay,
};

mod commands;
mod config;
mod utils;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    pub name: Option<String>,

    #[arg(short, long, value_name = "FILE", global = true)]
    pub config: Option<String>,

    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run tmux sessions
    Tmux,
    /// Manage workspaces
    Workspace(WorkspaceArgs),
}

#[derive(Debug, Args)]
struct WorkspaceArgs {
    #[command(subcommand)]
    pub command: WorkspaceCommands,
}

#[derive(Debug, Subcommand)]
enum WorkspaceCommands {
    /// List all known workspaces
    List(DisplayCommand),
    /// Get the current workspace
    Current(DisplayCommand),
}

#[derive(Debug, Args)]
struct DisplayCommand {
    /// Print json
    #[arg(long, default_value_if("json_pretty", "true", "true"))]
    pub json: bool,

    /// Print json, but pretty (implies --json)
    #[arg(long)]
    pub json_pretty: bool,
}

fn main() -> Result<(), io::Error> {
    let cli = Cli::parse();

    if let Some(name) = cli.name.as_deref() {
        println!("Value for name: {name}");
    }

    let config = load_config(cli.config)?;

    match &cli.command {
        Some(Commands::Tmux) => tmux(config),
        Some(Commands::Workspace(workspace_args)) => match &workspace_args.command {
            WorkspaceCommands::List(args) => list_workspaces(
                config,
                ListWorkspaceOptions {
                    display: &*create_display(args),
                },
            ),
            WorkspaceCommands::Current(args) => get_current_workspace(
                config,
                CurrentWorkspaceOptions {
                    display: &*create_display(args),
                },
            ),
        },
        None => {}
    }

    Ok(())
}

fn create_display(command: &DisplayCommand) -> Box<dyn WorkspaceDisplay> {
    let display: Box<dyn WorkspaceDisplay> = match command {
        DisplayCommand {
            json: true,
            json_pretty: false,
            ..
        } => Box::new(JsonWorkspaceDisplay {}),
        DisplayCommand {
            json: true,
            json_pretty: true,
            ..
        } => Box::new(JsonPrettyWorkspaceDisplay {}),
        DisplayCommand { json: false, .. } => Box::new(PrettyWorkspaceDisplay {}),
    };
    display
}
