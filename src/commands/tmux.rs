use std::{path::PathBuf, process::Command};

use crate::{config::{Config, Session, Tmux}, utils::path::expand_path_buf};

pub fn tmux(config: Config) {
    let sessions = match config.clone().tmux {
        None => tmux_none(config),
        Some(tmux_config) => tmux_some(config, tmux_config),
    };

    run_tmux(sessions);
}

fn tmux_none(config: Config) -> Vec<TmuxSession> {
    config
        .workspaces
        .into_iter()
        .map(|workspace| TmuxSession {
            path: expand_path_buf(&workspace.root),
            name: workspace.name,
            windows: vec![TmuxWindow {
                name: String::from("default"),
                command: None,
            }],
        })
        .collect()
}

fn tmux_some(config: Config, tmux: Tmux) -> Vec<TmuxSession> {
    tmux.sessions
        .into_iter()
        .map(|session| match session {
            Session::Workspace(workspace_session) => {
                let workspace = config
                    .workspaces
                    .iter()
                    .find(|w| w.id == workspace_session.workspace)
                    .unwrap();
                TmuxSession {
                    path: expand_path_buf(&workspace.root.clone()),
                    name: workspace_session.name.unwrap_or(workspace.name.clone()),
                    windows: workspace_session
                        .windows
                        .into_iter()
                        .map(|window| TmuxWindow {
                            name: window.name,
                            command: window.command,
                        })
                        .collect(),
                }
            }
            Session::Path(path_session) => TmuxSession {
                path: expand_path_buf(&path_session.path.clone()),
                name: path_session.name,
                windows: path_session
                    .windows
                    .into_iter()
                    .map(|window| TmuxWindow {
                        name: window.name,
                        command: window.command,
                    })
                    .collect(),
            },
        })
        .collect()
}

fn run_tmux(sessions: Vec<TmuxSession>) {
    for session in sessions {
        if session.windows.is_empty() {
            // Skip sessions with no windows
            continue;
        }

        // Create a new tmux session with the first window
        let first_window = &session.windows[0];
        let mut first_window_args = vec![
            "new-session",
            "-d",
            "-s",
            &session.name,
            "-n",
            &first_window.name,
        ];
        let command = command_with_shell(first_window.command.clone());
        if let Some(ref cmd) = command {
            first_window_args.push(cmd);
        }

        let session_creation = Command::new("tmux")
            .args(&first_window_args)
            .current_dir(&session.path)
            .output()
            .expect("Failed to create tmux session");

        if !session_creation.status.success() {
            eprintln!(
                "Failed to create session {}: {}",
                &session.name,
                String::from_utf8_lossy(&session_creation.stderr)
            );
            continue;
        }

        // Create the remaining windows
        for window in &session.windows[1..] {
            let mut window_args = vec!["new-window", "-t", &session.name, "-n", &window.name];
            let command = command_with_shell(window.command.clone());
            if let Some(ref cmd) = command {
                window_args.push(cmd);
            }

            let window_creation = Command::new("tmux")
                .args(&window_args)
                .output()
                .expect("Failed to create tmux window");

            if !window_creation.status.success() {
                eprintln!(
                    "Failed to create window {} in session {}: {}",
                    &window.name,
                    &session.name,
                    String::from_utf8_lossy(&window_creation.stderr)
                );
            }
        }
    }
}

fn command_with_shell(cmd: Option<String>) -> Option<String> {
    cmd.map(|cmd_str| cmd_str + "; exec $SHELL")
}

#[derive(Debug)]
struct TmuxSession {
    pub path: PathBuf,
    pub name: String,
    pub windows: Vec<TmuxWindow>,
}

#[derive(Debug)]
struct TmuxWindow {
    pub command: Option<String>,
    pub name: String,
}
