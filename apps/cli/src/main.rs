use std::sync::Arc;

use clap::{Args, CommandFactory, Parser, Subcommand};
use commands::{
    command::RafaeltabCommand,
    tmux::{
        list::{TmuxListRuntimeOptions},
        start::{TmuxStartRuntimeOptions},
        switch::TmuxSwitchRuntimeOptions,
    },
    workspaces::{
        add::WorkspaceAddRuntimeOptions,
        current::CurrentWorkspaceRuntimeOptions,
        find::FindWorkspaceRuntimeOptions,
        find_tag::FindTagWorkspaceRuntimeOptions,
        list::ListWorkspacesRuntimeOptions,
        tmux::ListTmuxWorkspacesRuntimeOptions,
    },
    worktree::{
        complete::WorktreeCompleteRuntimeOptions,
        start::WorktreeStartRuntimeOptions,
    },
};
use di::AppContainer;
use utils::display::{JsonDisplay, JsonPrettyDisplay, PrettyDisplay, RafaeltabDisplay};

#[allow(dead_code)]
mod commands;
#[allow(dead_code)]
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

fn main() -> Result<(), std::io::Error> {
    let cli = Cli::parse();

    // Resolve config path and create the DI container
    let config_path = resolve_config_path(cli.config)?;
    let container = AppContainer::new(Some(config_path))?;

    match &cli.command {
        Some(Commands::Tmux(tmux_args)) => match &tmux_args.command {
            TmuxCommands::List(args) => {
                let command = container.tmux_list_command();
                command
                    .execute(TmuxListRuntimeOptions {
                        json: args.json,
                        json_pretty: args.json_pretty,
                    })
                    .expect("Failed to execute tmux list command");
            }
            TmuxCommands::Start => {
                let command = container.tmux_start_command();
                command.execute(TmuxStartRuntimeOptions)
                    .expect("Failed to execute tmux start command");
            }
            TmuxCommands::Switch => {
                let command = container.tmux_switch_command();
                command.execute(TmuxSwitchRuntimeOptions)
                    .expect("Failed to execute tmux switch command");
            }
        },
        Some(Commands::Workspace(workspace_args)) => match &workspace_args.command {
            WorkspaceCommands::List(args) => {
                let command = container.list_workspaces_command();
                command.execute(ListWorkspacesRuntimeOptions {
                    json: args.json,
                    json_pretty: args.json_pretty,
                })
                .expect("Failed to execute workspace list command");
            }
            WorkspaceCommands::Current(args) => {
                let command = container.current_workspace_command();
                command.execute(CurrentWorkspaceRuntimeOptions {
                    json: args.json,
                    json_pretty: args.json_pretty,
                })
                .expect("Failed to execute current workspace command");
            }
            WorkspaceCommands::Find(args) => {
                let command = container.find_workspace_command();
                command.execute(FindWorkspaceRuntimeOptions {
                    id: args.id.clone(),
                    json: args.display_command.json,
                    json_pretty: args.display_command.json_pretty,
                })
                .expect("Failed to execute find workspace command");
            }
            WorkspaceCommands::FindTag(args) => {
                let command = container.find_tag_workspace_command();
                command
                    .execute(FindTagWorkspaceRuntimeOptions {
                        tag: args.tag.clone(),
                        json: args.display_command.json,
                        json_pretty: args.display_command.json_pretty,
                    })
                    .expect("Failed to execute find tag workspace command");
            }
            WorkspaceCommands::Tmux(args) => {
                let command = container.list_tmux_workspaces_command();
                command
                    .execute(ListTmuxWorkspacesRuntimeOptions {
                        json: args.json,
                        json_pretty: args.json_pretty,
                    })
                    .expect("Failed to execute list tmux workspaces command");
            }
            WorkspaceCommands::Add(args) => {
                let command = container.workspace_add_command();
                command.execute(WorkspaceAddRuntimeOptions {
                    json: args.display_command.json,
                    json_pretty: args.display_command.json_pretty,
                    interactive: args.interactive,
                    name: args.name.clone(),
                    tags: args.tags.clone(),
                    path: args.path.clone(),
                }).expect("Failed to execute workspace add command");
            }
        },
        Some(Commands::Worktree(worktree_args)) => match &worktree_args.command {
            WorktreeCommands::Start(args) => {
                let command = container.worktree_start_command();
                command.execute(WorktreeStartRuntimeOptions {
                    branch_name: args.branch_name.clone(),
                    force: args.force,
                    yes: args.yes,
                }).expect("Failed to execute worktree start command");
            }
            WorktreeCommands::Complete(args) => {
                let command = container.worktree_complete_command();
                command.execute(WorktreeCompleteRuntimeOptions {
                    branch_name: args.branch_name.clone(),
                    force: args.force,
                    yes: args.yes,
                }).expect("Failed to execute worktree complete command");
            }
        },
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
                    // Now we can use the DI container - no more Box::leak!
                    let workspace_repository = container.workspace_repository();

                    // Run the command palette
                    if palette.registry().is_empty() {
                        println!("No commands available");
                    } else {
                        // Create command context and run
                        use crate::commands::Command;
                        let mut ctx = crate::commands::CommandCtx::new(workspace_repository)
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

fn create_display_arc(command: &DisplayCommand) -> Arc<dyn RafaeltabDisplay> {
    let display: Arc<dyn RafaeltabDisplay> = match command {
        DisplayCommand {
            json: true,
            json_pretty: false,
            ..
        } => Arc::new(JsonDisplay {}),
        DisplayCommand {
            json: true,
            json_pretty: true,
            ..
        } => Arc::new(JsonPrettyDisplay {}),
        DisplayCommand { json: false, .. } => Arc::new(PrettyDisplay {}),
    };
    display
}

fn resolve_config_path(path: Option<String>) -> Result<String, std::io::Error> {
    use crate::utils::path::expand_path;
    use std::path::Path;

    static PATH_LOCATIONS_LINUX: &[&str] = &["~/.rafaeltab.json"];

    if let Some(path) = path {
        Ok(path)
    } else {
        // If config_path is not set, loop over PATH_LOCATIONS and find the first existing path
        for &path in PATH_LOCATIONS_LINUX {
            let full_path = expand_path(path);
            if Path::new(&full_path).exists() {
                return Ok(full_path);
            }
        }

        // If no existing path found, return an error
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No config file found in PATH_LOCATIONS",
        ))
    }
}
