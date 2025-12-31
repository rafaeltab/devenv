use serde::Deserialize;
use serde_json::json;

use crate::{
    domain::tmux_workspaces::{
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
    infrastructure::tmux_workspaces::tmux::{
        tmux_format::{TmuxFilterAstBuilder, TmuxFilterNode},
        tmux_format_variables::{TmuxFormatField, TmuxFormatVariable},
    },
    storage::tmux::TmuxStorage,
    utils::path::expand_path,
};

static TMUX_SESSION_ID_KEY: &str = "RAFAELTAB_SESSION_ID";

use super::tmux_client::TmuxRepository;

impl<TTmuxStorage> TmuxSessionRepository for TmuxRepository<'_, TTmuxStorage>
where
    TTmuxStorage: TmuxStorage,
{
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
        let full_path = expand_path(path);
        let mut args = vec![
            "new-session",
            "-d",
            "-P",
            "-F",
            &format,
            "-c",
            &full_path,
            "-e",
            &env,
            "-n",
            &first_window.name,
            "-s",
            &name,
        ];

        let first_command_with_shell = command_with_shell(first_window.command.clone());

        if let Some(ref command) = first_command_with_shell {
            args.push(command);
        }

        let session_id = self
            .connection
            .cmd(args)
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

        let session = sessions.first().unwrap().clone();

        for window in windows {
            self.new_window(&window.with_target(session.clone()).with_dir(&full_path));
        }

        session
    }

    fn kill_session(&self, session: Option<&TmuxSession>) {
        let mut args = vec!["kill-session"];
        if let Some(sess) = session {
            args.extend(["-t", &sess.id]);
        }
        self.connection
            .cmd(args)
            .stderr_to_stdout()
            .read()
            .expect("Failed to get sessions");
    }

    fn get_environment(&self, session_id: &str) -> String {
        self.connection
            .cmd(["show-environment", "-t", session_id])
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
        let result = self.connection.cmd(args).stderr_to_stdout().read();

        match result {
            Ok(res) => {
                let mut sessions: Vec<TmuxSession> = res
                    .lines()
                    .map(|x| {
                        serde_json::from_str::<ListSessionResponse>(x)
                            .expect("Failed to get sessions")
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
            Err(_) => vec![],
        }
    }
}

fn command_with_shell(cmd: Option<String>) -> Option<String> {
    cmd.map(|cmd_str| cmd_str + "; exec $SHELL")
}

#[derive(Deserialize)]
struct ListSessionResponse {
    path: String,
    name: String,
    id: String,
}
