//! Add Workspace command for the command palette.
//!
//! This command provides an interactive flow for adding a new workspace.

use std::env::current_dir;

use crate::commands::{Command, CommandCtx};
use crate::tui::picker_ctx::ExistingTagsSuggestionProvider;
use crate::utils::display::{PrettyDisplay, RafaeltabDisplay};

/// Command to add a new workspace.
///
/// This command guides the user through a multi-step process:
/// 1. Enter workspace name
/// 2. Enter tags (with suggestions from existing workspaces)
/// 3. Confirm the creation
///
/// # Example
///
/// ```ignore
/// use rafaeltab::commands::builtin::AddWorkspaceCommand;
/// use rafaeltab::commands::CommandCtx;
///
/// let cmd = AddWorkspaceCommand::new();
/// let mut ctx = CommandCtx::new().expect("Failed to create context");
/// cmd.run(&mut ctx);
/// ```
#[derive(Debug)]
pub struct AddWorkspaceCommand;

impl AddWorkspaceCommand {
    /// Create a new add workspace command.
    pub fn new() -> Self {
        Self
    }

    /// Slugify a name to create a valid workspace ID.
    ///
    /// Converts the name to lowercase and replaces special characters with hyphens.
    fn slugify(&self, name: &str) -> String {
        name.to_lowercase()
            .replace(|c: char| !c.is_alphanumeric(), "-")
            .replace("--", "-")
            .trim_matches('-')
            .to_string()
    }
}

impl Default for AddWorkspaceCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl AddWorkspaceCommand {
    fn inquire_name(&self, ctx: &mut CommandCtx) -> Option<String> {
        loop {
            let name = ctx.input("Workspace name")?;

            if !name.trim().is_empty() {
                return Some(name);
            }
        }
    }
}

impl Command for AddWorkspaceCommand {
    fn name(&self) -> &str {
        "Add Workspace"
    }

    fn description(&self) -> &str {
        "Create a workspace in the current directory"
    }

    fn run(&self, ctx: &mut CommandCtx) {
        // Step 1: Get workspace name (no suggestions)
        let name = match self.inquire_name(ctx) {
            Some(n) => n,
            _ => {
                // Empty name - cancel
                return;
            }
        };

        // Generate slugified ID from name
        let id = self.slugify(&name);

        // Step 2: Get tags (with suggestions from existing workspaces)
        // Collect all unique tags from existing workspaces
        let all_tags: Vec<String> = {
            let workspaces = ctx.workspace_repo().get_workspaces();
            let mut tags: Vec<String> = workspaces
                .iter()
                .flat_map(|w| w.tags.iter().map(|t| t.name.clone()))
                .collect();
            tags.sort();
            tags.dedup();
            tags
        };

        let tags_provider = ExistingTagsSuggestionProvider::new(all_tags);
        let tags_input =
            match ctx.input_with_suggestions("Tags (comma-separated)", Box::new(tags_provider)) {
                Some(t) => t,
                None => {
                    // User cancelled
                    return;
                }
            };

        // Parse, deduplicate, and sort tags
        let mut tags: Vec<String> = tags_input
            .split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect::<std::collections::HashSet<_>>() // Deduplicate
            .into_iter()
            .collect();
        tags.sort(); // Sort alphabetically for consistent ordering

        // Step 3: Confirm creation
        let tags_display = if tags.is_empty() {
            "(none)".to_string()
        } else {
            tags.join(", ")
        };

        let confirm_prompt = format!("Create workspace '{}' with tags [{}]?", name, tags_display);

        match ctx.confirm(&confirm_prompt, true) {
            Some(true) => {
                let _ = ctx.restore();

                let workspace = &ctx.workspace_repo().create_workspace(
                    name,
                    tags,
                    current_dir().unwrap().to_str().unwrap().to_string(),
                    id,
                );

                PrettyDisplay {}.display(workspace);
            }
            _ => {
                // User cancelled or selected No
            }
        }
    }
}
