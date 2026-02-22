use std::sync::Arc;

use duct::cmd;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    commands::{
        command::CommandError, command::RafaeltabCommand, tmux::legacy::TMUX_WORKSPACE_KEY,
    },
    domain::tmux_workspaces::{
        aggregates::workspaces::workspace::Workspace,
        repositories::workspace::workspace_repository::WorkspaceRepository,
    },
    utils::display::{DisplayFactory, RafaeltabDisplayItem, ToDynVec},
};

// Runtime options - CLI arguments only
pub struct ListTmuxWorkspacesRuntimeOptions {
    pub json: bool,
    pub json_pretty: bool,
}

// Command with injected dependencies
pub struct ListTmuxWorkspacesCommand {
    pub workspace_repository: Arc<dyn WorkspaceRepository>,
    pub display_factory: Arc<dyn DisplayFactory>,
}

impl RafaeltabCommand<ListTmuxWorkspacesRuntimeOptions> for ListTmuxWorkspacesCommand {
    fn execute(&self, options: ListTmuxWorkspacesRuntimeOptions) -> Result<(), CommandError> {
        // Create display from factory based on runtime options
        let display = self
            .display_factory
            .create_display(options.json, options.json_pretty);

        let format = json!({
            "name": "#{session_name}",
            "path": "#{session_path}",
        });

        let output = cmd!("tmux", "ls", "-F", format.to_string())
            .stderr_to_stdout()
            .read()
            .map_err(|e| CommandError::General(format!("Failed to list tmux sessions: {}", e)))?;

        let sessions: Vec<SessionOutput> = output
            .lines()
            .map(|x| {
                serde_json::from_str::<SessionOutput>(x).map_err(|e| {
                    CommandError::General(format!("Failed to parse session output: {}", e))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut results: Vec<SessionResult> = vec![];
        for session in &sessions {
            let session_env = cmd!("tmux", "show-environment", "-t", session.clone().name)
                .stderr_to_stdout()
                .read()
                .map_err(|e| {
                    CommandError::General(format!("Failed to get session environment: {}", e))
                })?;
            let workspace_line = session_env.lines().find(|x| x.contains(TMUX_WORKSPACE_KEY));
            results.push(match workspace_line {
                None => SessionResult {
                    session_name: session.name.clone(),
                    session_path: session.path.clone(),
                    workspace: None,
                },
                Some(line) => {
                    let workspace_id = line.split('=').next_back().ok_or_else(|| {
                        CommandError::General("Failed to parse workspace ID".to_string())
                    })?;
                    SessionResult {
                        session_name: session.name.clone(),
                        session_path: session.path.clone(),
                        workspace: self
                            .workspace_repository
                            .get_workspaces()
                            .into_iter()
                            .find(|x| x.id == workspace_id),
                    }
                }
            });
        }

        display.display_list(results.to_dyn_vec());
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct SessionOutput {
    name: String,
    path: String,
}

#[derive(Debug)]
struct SessionResult {
    session_name: String,
    session_path: String,
    workspace: Option<Workspace>,
}

impl RafaeltabDisplayItem for SessionResult {
    fn to_json(&self) -> Value {
        let mut val = json!({
            "session_name": self.session_name,
            "session_path": self.session_path,
        });

        if let Some(workspace) = &self.workspace {
            val["workspace"] = workspace.to_json();
        }
        json!(val)
    }

    fn to_pretty_string(&self) -> String {
        match &self.workspace {
            Some(workspace) => format!(
                "{} {} In workspace: {}",
                self.session_name,
                self.session_path,
                workspace.to_pretty_string()
            ),
            None => format!("{} {}", self.session_name, self.session_path),
        }
    }
}
