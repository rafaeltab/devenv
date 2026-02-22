use std::sync::Arc;

use serde::Deserialize;
use serde_json::json;
use shaku::Component;

use crate::domain::tmux_workspaces::aggregates::tmux::client::ClientIncludeFields;
use crate::domain::tmux_workspaces::repositories::tmux::client_repository::{
    SwitchClientTarget, TmuxClientRepository,
};
use crate::domain::tmux_workspaces::repositories::tmux::session_repository::TmuxSessionRepository;
use crate::infrastructure::tmux_workspaces::tmux::connection::TmuxConnectionInterface;
use crate::infrastructure::tmux_workspaces::tmux::tmux_format::{
    TmuxFilterAstBuilder, TmuxFilterNode,
};
use crate::{
    domain::tmux_workspaces::aggregates::tmux::client::TmuxClient,
    infrastructure::tmux_workspaces::tmux::tmux_format_variables::{
        TmuxFormatField, TmuxFormatVariable,
    },
};

#[derive(Component)]
#[shaku(interface = TmuxClientRepository)]
pub struct ImplClientRepository {
    #[shaku(inject)]
    pub connection: Arc<dyn TmuxConnectionInterface>,
    #[shaku(inject)]
    pub session_repository: Arc<dyn TmuxSessionRepository>,
}

impl TmuxClientRepository for ImplClientRepository {
    fn get_clients(
        &self,
        filter: Option<TmuxFilterNode>,
        include: ClientIncludeFields,
    ) -> Vec<TmuxClient> {
        let list_format = json!({
            "name": TmuxFormatVariable::ClientName.to_format(),
            "session": TmuxFormatVariable::ClientSession.to_format(),
        });

        let mut args = vec![
            "list-clients".to_string(),
            "-F".to_string(),
            list_format.to_string(),
        ];

        args.extend(match filter {
            Some(f) => vec!["-f".to_string(), f.as_string()],
            None => vec![],
        });

        let res = self
            .connection
            .cmd_owned(&args)
            .stderr_to_stdout()
            .read()
            .expect("Failed to get clients");

        let responses: Vec<ListClientResponse> = res
            .lines()
            .map(|x| serde_json::from_str::<ListClientResponse>(x).expect("Failed to get clients"))
            .collect();
        match include.clone().attached_to {
            Some(session_includes) => {
                let sessions = self
                    .session_repository
                    .get_sessions(Some(filter_responses(&responses)), session_includes);
                responses
                    .iter()
                    .map(|x| TmuxClient {
                        name: x.name.clone(),
                        attached_to: sessions.iter().find(|y| y.name == x.session).cloned(),
                        include_fields: include.clone(),
                    })
                    .collect()
            }
            None => responses
                .iter()
                .map(|x| TmuxClient {
                    name: x.name.clone(),
                    attached_to: None,
                    include_fields: include.clone(),
                })
                .collect(),
        }
    }

    fn switch_client(&self, client: Option<&TmuxClient>, target: SwitchClientTarget) {
        let target_id = match target {
            SwitchClientTarget::Session(session) => &session.id,
            SwitchClientTarget::Window(window) => &window.id,
            SwitchClientTarget::Pane(pane) => &pane.id,
        };
        let mut args = vec!["switch-client".to_string()];
        if let Some(c) = client {
            args.extend(["-c".to_string(), c.name.clone()]);
        }
        args.extend(["-t".to_string(), target_id.to_string()]);
        self.connection
            .cmd_owned(&args)
            .run()
            .expect("Unable to switch client");
    }
}

#[derive(Deserialize, Debug)]
struct ListClientResponse {
    name: String,
    session: String,
}

fn filter_responses(responses: &[ListClientResponse]) -> TmuxFilterNode {
    TmuxFilterAstBuilder::build(|b| {
        b.any(
            responses
                .iter()
                .map(|x| {
                    b.eq(
                        b.var(TmuxFormatVariable::SessionName),
                        b.const_val(&x.session),
                    )
                })
                .collect(),
        )
    })
}
