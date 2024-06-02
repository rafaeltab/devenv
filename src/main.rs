use std::io;

use clap::{Args, CommandFactory, Parser, Subcommand};
use commands::{
    tmux::tmux,
    workspaces::{
        current::{get_current_workspace, CurrentWorkspaceOptions},
        find::{find_workspace, FindWorkspaceOptions},
        find_tag::{find_tag_workspace, FindTagWorkspaceOptions},
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
#[command(version, about, long_about = None, disable_help_subcommand(true))]
struct Cli {
    #[arg(short, long, value_name = "FILE", global = true)]
    pub config: Option<String>,

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
    /// Find a specific workspace using an id
    Find(FindCommand),
    /// Find workspaces that have a tag
    FindTag(FindTagCommand),
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

#[derive(Debug, Args)]
struct FindCommand {
    #[command(flatten)]
    display_command: DisplayCommand,

    #[arg()]
    id: String,
}

#[derive(Debug, Args)]
struct FindTagCommand {
    #[command(flatten)]
    display_command: DisplayCommand,

    #[arg()]
    tag: String,
}

fn main() -> Result<(), io::Error> {
    let cli = Cli::parse();

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
            WorkspaceCommands::Find(args) => find_workspace(
                config,
                &args.id,
                FindWorkspaceOptions {
                    display: &*create_display(&args.display_command),
                },
            ),
            WorkspaceCommands::FindTag(args) => find_tag_workspace(
                config,
                &args.tag,
                FindTagWorkspaceOptions {
                    display: &*create_display(&args.display_command),
                },
            ),
        },
        None => {
            let _ = Cli::command().print_help();
        }
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
