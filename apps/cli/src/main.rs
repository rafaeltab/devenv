// #![feature(coroutines, coroutine_trait)]
// #![feature(stmt_expr_attributes)]
use std::io;

use clap::{Args, CommandFactory, Parser, Subcommand};
use commands::{
    command::RafaeltabCommand,
    tmux::{
        list::{TmuxListCommand, TmuxListOptions},
        start::{TmuxStartCommand, TmuxStartOptions},
    },
    workspaces::{
        add::{WorkspaceAddCommand, WorkspaceAddOptions},
        current::{get_current_workspace, CurrentWorkspaceOptions},
        find::{find_workspace_cmd, FindWorkspaceOptions},
        find_tag::{find_tag_workspace, FindTagWorkspaceOptions},
        list::{ListWorkspacesCommand, ListWorkspacesCommandArgs},
        tmux::{list_tmux_workspaces, ListTmuxWorkspaceOptions},
    },
    worktree::{
        complete::{WorktreeCompleteCommand, WorktreeCompleteOptions},
        start::{WorktreeStartCommand, WorktreeStartOptions},
    },
};
use infrastructure::tmux_workspaces::{
    repositories::{
        tmux::{description_repository::ImplDescriptionRepository, tmux_client::TmuxRepository},
        workspace::workspace_repository::ImplWorkspaceRepository,
    },
    tmux::connection::TmuxConnection,
};
use storage::kinds::json_storage::JsonStorageProvider;
use utils::display::{JsonDisplay, JsonPrettyDisplay, PrettyDisplay, RafaeltabDisplay};

use crate::commands::tmux::switch::{TmuxSwitchCommand, TmuxSwitchOptions};

#[allow(dead_code)]
mod commands;
#[allow(dead_code)]
mod domain;
#[allow(dead_code)]
mod infrastructure;
#[allow(dead_code)]
mod storage;
#[allow(dead_code)]
mod tui;
#[allow(dead_code)]
mod utils;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, disable_help_subcommand(true))]
struct Cli {
    /// Path to configuration file (defaults to ~/.rafaeltab.json)
    #[arg(short, long, value_name = "FILE", global = true)]
    pub config: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Open the command palette
    CommandPalette(CommandPaletteArgs),
    /// Run tmux sessions
    Tmux(TmuxArgs),
    /// Manage workspaces
    Workspace(WorkspaceArgs),
    /// Manage git worktrees
    Worktree(WorktreeArgs),
}

#[derive(Debug, Args)]
struct CommandPaletteArgs {
    #[command(subcommand)]
    pub command: CommandPaletteCommands,
}

#[derive(Debug, Subcommand)]
enum CommandPaletteCommands {
    /// Show the command palette
    Show,
}

#[derive(Debug, Args)]
struct TmuxArgs {
    #[command(subcommand)]
    pub command: TmuxCommands,
}

#[derive(Debug, Subcommand)]
enum TmuxCommands {
    /// List all tmux sessions with descriptions
    List(DisplayCommand),
    /// Start a new tmux session interactively
    Start,
    /// Switch to a different tmux session
    Switch,
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
    /// Add a new workspace
    Add(AddCommand),
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
struct AddCommand {
    #[command(flatten)]
    display_command: DisplayCommand,

    /// Name of the workspace
    #[arg(long)]
    name: Option<String>,

    /// Tags to associate with the workspace
    #[arg(long)]
    tags: Option<Vec<String>>,

    /// Path to the workspace directory
    #[arg(long)]
    path: Option<String>,

    /// Run in interactive mode
    #[arg(long)]
    interactive: Option<bool>,
}

#[derive(Debug, Args)]
struct FindCommand {
    #[command(flatten)]
    display_command: DisplayCommand,

    /// Workspace identifier to search for
    #[arg()]
    id: String,
}

#[derive(Debug, Args)]
struct FindTagCommand {
    #[command(flatten)]
    display_command: DisplayCommand,

    /// Tag name to search for
    #[arg()]
    tag: String,
}

#[derive(Debug, Args)]
struct WorktreeArgs {
    #[command(subcommand)]
    pub command: WorktreeCommands,
}

#[derive(Debug, Subcommand)]
enum WorktreeCommands {
    /// Start a new worktree for a branch
    Start(WorktreeStartArgs),
    /// Complete (remove) a worktree
    Complete(WorktreeCompleteArgs),
}

#[derive(Debug, Args)]
struct WorktreeStartArgs {
    /// The branch name for the new worktree
    #[arg()]
    branch_name: String,

    /// Force creation even without worktree config
    #[arg(long)]
    force: bool,

    /// Skip confirmation prompt
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(Debug, Args)]
struct WorktreeCompleteArgs {
    /// The branch name of the worktree to complete (defaults to current directory)
    #[arg()]
    branch_name: Option<String>,

    /// Force removal even with uncommitted/unpushed changes
    #[arg(long)]
    force: bool,

    /// Skip confirmation prompt
    #[arg(short = 'y', long)]
    yes: bool,
}

fn main() -> Result<(), io::Error> {
    let cli = Cli::parse();

    let storage_provider = JsonStorageProvider::new(cli.config)?;
    let storage = storage_provider.load()?;

    // Support test isolation via environment variable
    let tmux_connection = match std::env::var("RAFAELTAB_TMUX_SOCKET") {
        Ok(socket) => TmuxConnection::with_socket(socket),
        Err(_) => TmuxConnection::default(),
    };

    match &cli.command {
        Some(Commands::Tmux(tmux_args)) => match &tmux_args.command {
            TmuxCommands::List(args) => TmuxListCommand.execute(TmuxListOptions {
                display: &*create_display(args),
                session_description_repository: &ImplDescriptionRepository {
                    workspace_repository: &ImplWorkspaceRepository {
                        workspace_storage: &storage,
                    },
                    session_repository: &TmuxRepository {
                        tmux_storage: &storage,
                        connection: &tmux_connection,
                    },
                    tmux_storage: &storage,
                },
            }),
            TmuxCommands::Start => {
                let session_repository = &TmuxRepository {
                    tmux_storage: &storage,
                    connection: &tmux_connection,
                };
                TmuxStartCommand.execute(TmuxStartOptions {
                    session_description_repository: &ImplDescriptionRepository {
                        workspace_repository: &ImplWorkspaceRepository {
                            workspace_storage: &storage,
                        },
                        session_repository,
                        tmux_storage: &storage,
                    },
                    session_repository,
                    tmux_storage: &storage,
                })
            }
            TmuxCommands::Switch => {
                let tmux_repository = &TmuxRepository {
                    tmux_storage: &storage,
                    connection: &tmux_connection,
                };
                TmuxSwitchCommand.execute(TmuxSwitchOptions {
                    session_description_repository: &ImplDescriptionRepository {
                        workspace_repository: &ImplWorkspaceRepository {
                            workspace_storage: &storage,
                        },
                        session_repository: tmux_repository,
                        tmux_storage: &storage,
                    },
                    session_repository: tmux_repository,
                    client_repository: tmux_repository,
                    tmux_storage: &storage,
                })
            }
        },
        Some(Commands::Workspace(workspace_args)) => match &workspace_args.command {
            WorkspaceCommands::List(args) => {
                ListWorkspacesCommand.execute(ListWorkspacesCommandArgs {
                    workspace_storage: &storage,
                    display: &*create_display(args),
                })
            }
            WorkspaceCommands::Current(args) => get_current_workspace(
                &storage,
                CurrentWorkspaceOptions {
                    display: &*create_display(args),
                },
            ),
            WorkspaceCommands::Find(args) => find_workspace_cmd(
                &storage,
                &args.id,
                FindWorkspaceOptions {
                    display: &*create_display(&args.display_command),
                },
            ),
            WorkspaceCommands::FindTag(args) => find_tag_workspace(
                &storage,
                &args.tag,
                FindTagWorkspaceOptions {
                    display: &*create_display(&args.display_command),
                },
            ),
            WorkspaceCommands::Tmux(args) => list_tmux_workspaces(
                &storage,
                ListTmuxWorkspaceOptions {
                    display: &*create_display(args),
                },
            ),
            WorkspaceCommands::Add(args) => {
                let workspace_repository = ImplWorkspaceRepository {
                    workspace_storage: &storage,
                };
                WorkspaceAddCommand.execute(WorkspaceAddOptions {
                    display: &*create_display(&args.display_command),
                    workspace_repository: &workspace_repository,
                    interactive: args.interactive,
                    name: args.name.clone(),
                    tags: args.tags.clone(),
                    path: args.path.clone(),
                })
            }
        },
        Some(Commands::Worktree(worktree_args)) => {
            let tmux_repository = &TmuxRepository {
                tmux_storage: &storage,
                connection: &tmux_connection,
            };
            let workspace_repository = &ImplWorkspaceRepository {
                workspace_storage: &storage,
            };

            match &worktree_args.command {
                WorktreeCommands::Start(args) => {
                    WorktreeStartCommand.execute(WorktreeStartOptions {
                        branch_name: args.branch_name.clone(),
                        force: args.force,
                        yes: args.yes,
                        workspace_repository,
                        worktree_storage: &storage,
                        session_repository: tmux_repository,
                        client_repository: tmux_repository,
                        tmux_storage: &storage,
                    })
                }
                WorktreeCommands::Complete(args) => {
                    let description_repository = &ImplDescriptionRepository {
                        workspace_repository,
                        session_repository: tmux_repository,
                        tmux_storage: &storage,
                    };
                    let popup_repository = &infrastructure::tmux_workspaces::repositories::tmux::popup_repository::ImplPopupRepository {
                        connection: &tmux_connection,
                    };

                    WorktreeCompleteCommand.execute(WorktreeCompleteOptions {
                        branch_name: args.branch_name.clone(),
                        force: args.force,
                        yes: args.yes,
                        workspace_repository,
                        session_repository: tmux_repository,
                        client_repository: tmux_repository,
                        popup_repository,
                        description_repository,
                    })
                }
            }
        }
        Some(Commands::CommandPalette(palette_args)) => {
            use crate::commands::{
                builtin::AddWorkspaceCommand, registry::CommandRegistry, CommandPalette,
                TestConfirmCommand, TestPickerCommand, TestTextInputCommand,
                TestTextInputSuggestionsCommand,
            };

            // Create command registry
            let mut registry = CommandRegistry::new();

            // Register normal commands
            registry.register(AddWorkspaceCommand::new());

            // Register test commands only in TEST_MODE
            if std::env::var("TEST_MODE").is_ok() {
                registry.register(TestPickerCommand::new());
                registry.register(TestTextInputCommand::new());
                registry.register(TestTextInputSuggestionsCommand::new());
                registry.register(TestConfirmCommand::new());
            }

            // Create the command palette
            let palette = CommandPalette::new(registry);

            // Handle subcommands
            match &palette_args.command {
                CommandPaletteCommands::Show => {
                    // Run the command palette
                    if palette.registry().is_empty() {
                        println!("No commands available");
                    } else {
                        // Create command context and run
                        use crate::commands::Command;
                        let mut ctx = crate::commands::CommandCtx::new()
                            .expect("Failed to create command context");
                        palette.run(&mut ctx);
                    }
                }
            }
        }
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
