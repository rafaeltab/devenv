use std::io;

use clap::{Args, CommandFactory, Parser, Subcommand};
use commands::{
    command::RafaeltabCommand,
    tmux::{
        // legacy::{TmuxCommand, TmuxCommandArgs},
        list::{TmuxListCommand, TmuxListOptions},
        start::{TmuxStartCommand, TmuxStartOptions},
    },
    workspaces::{
        current::{get_current_workspace, CurrentWorkspaceOptions},
        find::{find_workspace_cmd, FindWorkspaceOptions},
        find_tag::{find_tag_workspace, FindTagWorkspaceOptions},
        list::{ListWorkspacesCommand, ListWorkspacesCommandArgs},
        tmux::{list_tmux_workspaces, ListTmuxWorkspaceOptions},
    },
};
use config::load_config;
use infrastructure::repositories::{
    tmux::{description_repository::ImplDescriptionRepository, tmux_client::TmuxRepository},
    workspace::workspace_repository::ImplWorkspaceRepository,
};
use utils::display::{JsonDisplay, JsonPrettyDisplay, PrettyDisplay, RafaeltabDisplay};

mod commands;
mod config;
mod domain;
mod infrastructure;
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
    Tmux(TmuxArgs),
    /// Manage workspaces
    Workspace(WorkspaceArgs),
}

#[derive(Debug, Args)]
struct TmuxArgs {
    #[command(subcommand)]
    pub command: TmuxCommands,
}

#[derive(Debug, Subcommand)]
enum TmuxCommands {
    List(DisplayCommand),
    Start,
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
    /// List tmux sessions, with their attached workspaces
    Tmux(DisplayCommand),
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
    // let repo = TmuxRepository {};
    // let includes = IncludeFieldsBuilder::new()
    //     .with_panes(true)
    //     .with_windows(true)
    //     .with_environment(true)
    //     .with_attached_to(true);
    // let sessions = repo.get_sessions(None, includes.build_session());
    //
    // println!("{:#?}", sessions);
    //
    // let a = false;
    // if !a {
    //     return Ok(());
    // }
    let cli = Cli::parse();

    let config = load_config(cli.config)?;

    match &cli.command {
        Some(Commands::Tmux(tmux_args)) => match &tmux_args.command {
            TmuxCommands::List(args) => TmuxListCommand.execute(TmuxListOptions {
                display: &*create_display(args),
                session_description_repository: &ImplDescriptionRepository {
                    workspace_repository: &ImplWorkspaceRepository {
                        config: config.clone(),
                    },
                    session_repository: &TmuxRepository {
                        config: config.clone(),
                    },
                    config: config.clone(),
                },
            }),
            TmuxCommands::Start => {
                let session_repository = &TmuxRepository {
                    config: config.clone(),
                };
                TmuxStartCommand.execute(TmuxStartOptions {
                    session_description_repository: &ImplDescriptionRepository {
                        workspace_repository: &ImplWorkspaceRepository {
                            config: config.clone(),
                        },
                        session_repository,
                        config: config.clone(),
                    },
                    session_repository,
                })
            }
        },
        Some(Commands::Workspace(workspace_args)) => match &workspace_args.command {
            WorkspaceCommands::List(args) => {
                ListWorkspacesCommand.execute(ListWorkspacesCommandArgs {
                    config,
                    display: &*create_display(args),
                })
            }
            WorkspaceCommands::Current(args) => get_current_workspace(
                config,
                CurrentWorkspaceOptions {
                    display: &*create_display(args),
                },
            ),
            WorkspaceCommands::Find(args) => find_workspace_cmd(
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
            WorkspaceCommands::Tmux(args) => list_tmux_workspaces(
                config,
                ListTmuxWorkspaceOptions {
                    display: &*create_display(args),
                },
            ),
        },
        None => {
            let _ = Cli::command().print_help();
        }
    }

    Ok(())
}

fn create_display(command: &DisplayCommand) -> Box<dyn RafaeltabDisplay> {
    let display: Box<dyn RafaeltabDisplay> = match command {
        DisplayCommand {
            json: true,
            json_pretty: false,
            ..
        } => Box::new(JsonDisplay {}),
        DisplayCommand {
            json: true,
            json_pretty: true,
            ..
        } => Box::new(JsonPrettyDisplay {}),
        DisplayCommand { json: false, .. } => Box::new(PrettyDisplay {}),
    };
    display
}
