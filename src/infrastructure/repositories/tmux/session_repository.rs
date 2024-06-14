use duct::cmd;
use serde::Deserialize;
use serde_json::json;

use crate::{
    domain::{
        aggregates::tmux::session::{SessionIncludeFields, TmuxSession},
        repositories::tmux::{
            session_repository::TmuxSessionRepository,
            window_repository::{GetWindowsTarget, TmuxWindowRepository},
        },
    },
    infrastructure::tmux::{
        tmux_format::TmuxFilterNode,
        tmux_format_variables::{TmuxFormatField, TmuxFormatVariable},
    },
};

use super::tmux_client::TmuxRepository;

impl TmuxSessionRepository for TmuxRepository {
    fn new_session(&self) -> TmuxSession {
        todo!()
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

    fn get_sessions(
        &self,
        filter: Option<TmuxFilterNode>,
        include: SessionIncludeFields,
    ) -> Vec<TmuxSession> {
        let list_format = json!({
            "id": TmuxFormatVariable::SessionId.to_format(),
            "name": TmuxFormatVariable::SessionName.to_format(),
            "window_count": TmuxFormatVariable::SessionWindows.to_format(),
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
                name: x.name,
                id: x.id,
                windows: None,
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

        sessions
    }
}

#[derive(Deserialize)]
struct ListSessionResponse {
    name: String,
    id: String,
}
