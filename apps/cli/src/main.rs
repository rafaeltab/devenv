// #![feature(coroutines, coroutine_trait)]
// #![feature(stmt_expr_attributes)]
use std::{io, sync::Arc};

use clap::{Args, CommandFactory, Parser, Subcommand};
use commands::{
    tmux::{
        list::TmuxListCommandInterface,
        start::TmuxStartCommandInterface,
        switch::TmuxSwitchCommandInterface,
    },
    workspaces::{
        add::WorkspaceAddCommandInterface,
        current::CurrentWorkspaceCommandInterface,
        find::FindWorkspaceCommandInterface,
        find_tag::FindTagWorkspaceCommandInterface,
        list::ListWorkspacesCommandInterface,
        tmux::ListTmuxWorkspacesCommandInterface,
    },
    worktree::{
        complete::WorktreeCompleteCommandInterface,
        start::WorktreeStartCommandInterface,
    },
};
use infrastructure::tmux_workspaces::tmux::connection::TmuxConnection;
use shaku::HasComponent;
use storage::kinds::json_storage::JsonStorageProvider;
use utils::display::{JsonDisplay, JsonPrettyDisplay, PrettyDisplay, RafaeltabDisplay};

#[allow(dead_code)]
mod commands;
mod di;
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
    let storage = Arc::new(storage_provider.load()?);

    // Support test isolation via environment variable
    let tmux_connection: Box<TmuxConnection> = Box::new(match std::env::var("RAFAELTAB_TMUX_SOCKET") {
        Ok(socket) => TmuxConnection::with_socket(socket),
        Err(_) => TmuxConnection::default(),
    });

    // Build the DI module with component overrides for storage wrappers and TmuxConnection
    let module = di::AppModule::builder()
        .with_component_override::<dyn storage::workspace::WorkspaceStorage>(
            Box::new(storage::kinds::json_storage::JsonWorkspaceStorage::new(storage.clone())),
        )
        .with_component_override::<dyn storage::tmux::TmuxStorage>(
            Box::new(storage::kinds::json_storage::JsonTmuxStorage::new(storage.clone())),
        )
        .with_component_override::<dyn storage::worktree::WorktreeStorage>(
            Box::new(storage::kinds::json_storage::JsonWorktreeStorage::new(storage.clone())),
        )
        .with_component_override::<dyn infrastructure::tmux_workspaces::tmux::connection::TmuxConnectionInterface>(
            tmux_connection,
        )
        .build();

    match &cli.command {
        Some(Commands::Tmux(tmux_args)) => match &tmux_args.command {
            TmuxCommands::List(args) => {
                let cmd: &dyn TmuxListCommandInterface = module.resolve_ref();
                cmd.execute(commands::tmux::list::TmuxListArgs {
                    display: &*create_display(args),
                });
            }
            TmuxCommands::Start => {
                let cmd: &dyn TmuxStartCommandInterface = module.resolve_ref();
                cmd.execute();
            }
            TmuxCommands::Switch => {
                let cmd: &dyn TmuxSwitchCommandInterface = module.resolve_ref();
                cmd.execute();
            }
        },
        Some(Commands::Workspace(workspace_args)) => match &workspace_args.command {
            WorkspaceCommands::List(args) => {
                let cmd: &dyn ListWorkspacesCommandInterface = module.resolve_ref();
                cmd.execute(commands::workspaces::list::ListWorkspacesArgs {
                    display: &*create_display(args),
                });
            }
            WorkspaceCommands::Current(args) => {
                let cmd: &dyn CurrentWorkspaceCommandInterface = module.resolve_ref();
                cmd.execute(commands::workspaces::current::CurrentWorkspaceArgs {
                    display: &*create_display(args),
                });
            }
            WorkspaceCommands::Find(args) => {
                let cmd: &dyn FindWorkspaceCommandInterface = module.resolve_ref();
                cmd.execute(commands::workspaces::find::FindWorkspaceArgs {
                    display: &*create_display(&args.display_command),
                    id: args.id.clone(),
                });
            }
            WorkspaceCommands::FindTag(args) => {
                let cmd: &dyn FindTagWorkspaceCommandInterface = module.resolve_ref();
                cmd.execute(commands::workspaces::find_tag::FindTagWorkspaceArgs {
                    display: &*create_display(&args.display_command),
                    tag: args.tag.clone(),
                });
            }
            WorkspaceCommands::Tmux(args) => {
                let cmd: &dyn ListTmuxWorkspacesCommandInterface = module.resolve_ref();
                cmd.execute(commands::workspaces::tmux::ListTmuxWorkspacesArgs {
                    display: &*create_display(args),
                });
            }
            WorkspaceCommands::Add(args) => {
                let cmd: &dyn WorkspaceAddCommandInterface = module.resolve_ref();
                cmd.execute(commands::workspaces::add::WorkspaceAddArgs {
                    display: &*create_display(&args.display_command),
                    interactive: args.interactive,
                    name: args.name.clone(),
                    tags: args.tags.clone(),
                    path: args.path.clone(),
                });
            }
        },
        Some(Commands::Worktree(worktree_args)) => {
            match &worktree_args.command {
                WorktreeCommands::Start(args) => {
                    let cmd: &dyn WorktreeStartCommandInterface = module.resolve_ref();
                    cmd.execute(commands::worktree::start::WorktreeStartArgs {
                        branch_name: args.branch_name.clone(),
                        force: args.force,
                        yes: args.yes,
                    });
                }
                WorktreeCommands::Complete(args) => {
                    let cmd: &dyn WorktreeCompleteCommandInterface = module.resolve_ref();
                    cmd.execute(commands::worktree::complete::WorktreeCompleteArgs {
                        branch_name: args.branch_name.clone(),
                        force: args.force,
                        yes: args.yes,
                    });
                }
            }
        }
        Some(Commands::CommandPalette(palette_args)) => {
            match &palette_args.command {
                CommandPaletteCommands::Show => {
                    let cmd: &dyn commands::command_palette::CommandPaletteInterface =
                        module.resolve_ref();
                    cmd.execute();
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
