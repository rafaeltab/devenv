use duct::cmd;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    commands::tmux::TMUX_WORKSPACE_KEY, config::{Config, Workspace}, utils::workspace::{self, find_workspace, RafaeltabDisplay, RafaeltabDisplayItem, ToDynVec}
};

pub struct ListTmuxWorkspaceOptions<'a> {
    pub display: &'a dyn RafaeltabDisplay,
}

pub fn list_tmux_workspaces(config: Config, ListTmuxWorkspaceOptions { display }: ListTmuxWorkspaceOptions) {
    let format = json!({
        "name": "#{session_name}",
        "path": "#{session_path}",
    });

    let output = cmd!("tmux", "ls", "-F", format.to_string())
        .stderr_to_stdout()
        .read()
        .unwrap();
    let sessions: Vec<SessionOutput> = output
        .lines()
        .map(|x| serde_json::from_str::<SessionOutput>(x).unwrap())
        .collect();

    let mut results: Vec<SessionResult> = vec![];
    for session in &sessions {
        let session_env = cmd!("tmux", "show-environment", "-t", session.clone().name)
            .stderr_to_stdout()
            .read()
            .unwrap();
        let workspace_line = session_env.lines().find(|x| x.contains(TMUX_WORKSPACE_KEY));
        results.push(match workspace_line {
            None => SessionResult {
                session_name: session.name.clone(),
                session_path: session.path.clone(),
                workspace: None,
            },
            Some(line) => {
                let workspace_id = line.split('=').last().unwrap();
                SessionResult {
                    session_name: session.name.clone(),
                    session_path: session.path.clone(),
                    workspace: find_workspace(&config, workspace_id),
                }
            }
        });
    }

    display.display_list(results.to_dyn_vec());
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
            Some(workspace) => format!("{} {} In workspace: {}", self.session_name, self.session_path, workspace.to_pretty_string()),
            None => format!("{} {}", self.session_name, self.session_path),
        }
    }
}
