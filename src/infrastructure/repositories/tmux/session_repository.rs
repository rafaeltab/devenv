use duct::cmd;
use serde::Deserialize;
use serde_json::json;

use crate::{
    domain::{
        aggregates::tmux::{
            description::{
                session::{SessionDescription, SessionKind},
                window::WindowDescription,
            },
            include_fields_builder::IncludeFieldsBuilder,
            session::{SessionIncludeFields, TmuxSession},
        },
        repositories::tmux::{
            session_repository::TmuxSessionRepository,
            window_repository::{GetWindowsTarget, NewWindowBuilder, TmuxWindowRepository},
        },
    },
    infrastructure::tmux::{
        tmux_format::{TmuxFilterAstBuilder, TmuxFilterNode},
        tmux_format_variables::{TmuxFormatField, TmuxFormatVariable},
    },
};

static TMUX_SESSION_ID_KEY: &str = "RAFAELTAB_SESSION_ID";

use super::tmux_client::TmuxRepository;

impl TmuxSessionRepository for TmuxRepository {
    fn new_session(&self, description: &SessionDescription) -> TmuxSession {
        let name = &description.name;
        let path = match &description.kind {
            SessionKind::Path(path) => &path.path,
            SessionKind::Workspace(workspace) => &workspace.path,
        };
        let id = &description.id;

        let mut windows: Vec<NewWindowBuilder> = vec![];

        for window in description.windows.clone().iter().skip(1) {
            let builder = NewWindowBuilder::new()
                .with_dir(path.clone())
                .with_name(window.name.clone());
            windows.push(builder);
        }
        let default_description = WindowDescription {
            command: None,
            name: "zsh".to_string(),
        };
        let first_window = description.windows.first().unwrap_or(&default_description);
        let env = format!("{}={}", TMUX_SESSION_ID_KEY, id);
        let format = TmuxFormatVariable::SessionId.to_format();
        let mut args = vec![
            "new-session",
            "-d",
            "-P",
            "-F",
            &format,
            "-c",
            &path,
            "-e",
            &env,
            "-n",
            &first_window.name,
            "-s",
            &name,
        ];

        if let Some(ref command) = first_window.command {
            args.push(command);
        }

        let session_id = cmd("tmux", args)
            .stderr_to_stdout()
            .read()
            .expect("Expected to succeed creating session");

        let sessions = self.get_sessions(
            Some(TmuxFilterAstBuilder::build(|b| {
                b.eq(
                    b.const_val(&session_id),
                    b.var(TmuxFormatVariable::SessionId),
                )
            })),
            IncludeFieldsBuilder::new().build_session(),
        );

        sessions.first().unwrap().clone()
    }

    fn kill_session(&self, session: Option<&TmuxSession>) {
        let mut args = vec!["kill-session"];
        if let Some(sess) = session {
            args.extend(["-t", &sess.id]);
        }
        cmd("tmux", args)
            .stderr_to_stdout()
            .read()
            .expect("Failed to get sessions");
    }

    fn get_environment(&self, session_id: &str) -> String {
        cmd!("tmux", "show-environment", "-t", session_id)
            .stderr_to_stdout()
            .read()
            .expect("Failed to get sessions")
    }

    fn get_sessions(
        &self,
        filter: Option<TmuxFilterNode>,
        include: SessionIncludeFields,
    ) -> Vec<TmuxSession> {
        let list_format = json!({
            "id": TmuxFormatVariable::SessionId.to_format(),
            "name": TmuxFormatVariable::SessionName.to_format(),
            "path": TmuxFormatVariable::SessionPath.to_format(),
        })
        .to_string();
        let mut args = vec!["list-sessions", "-F", &list_format];
        let filter_string = match filter.map(|x| x.as_string()) {
            Some(val) => val,
            None => "".to_string(),
        };
        if !filter_string.is_empty() {
            args.extend(["-f", &filter_string]);
        }
        let res = cmd("tmux", args)
            .stderr_to_stdout()
            .read()
            .expect("Failed to get sessions");
        let mut sessions: Vec<TmuxSession> = res
            .lines()
            .map(|x| {
                serde_json::from_str::<ListSessionResponse>(x).expect("Failed to get sessions")
            })
            .map(|x| TmuxSession {
                id: x.id,
                name: x.name,
                path: x.path,
                windows: None,
                environment: None,
                include_fields: include.clone(),
            })
            .collect();
        if let Some(window_includes) = include.windows {
            (0..sessions.len()).for_each(|i| {
                let windows = self.get_windows(
                    None,
                    window_includes.clone(),
                    GetWindowsTarget::Session {
                        id: &sessions[i].id,
                    },
                );
                sessions[i].windows = Some(windows);
            });
        }
        if let Some(()) = include.environment {
            (0..sessions.len()).for_each(|i| {
                let environment = self.get_environment(&sessions[i].id);
                sessions[i].environment = Some(environment);
            });
        }

        sessions
    }
}

#[derive(Deserialize)]
struct ListSessionResponse {
    path: String,
    name: String,
    id: String,
}
