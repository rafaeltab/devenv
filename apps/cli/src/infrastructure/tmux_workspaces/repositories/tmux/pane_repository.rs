use std::collections::HashSet;
use std::sync::Arc;

use serde::Deserialize;
use serde_json::json;
use shaku::Component;

use crate::{
    domain::tmux_workspaces::{
        aggregates::tmux::pane::TmuxPane,
        repositories::tmux::pane_repository::{GetPanesTarget, SplitDirection, TmuxPaneRepository},
    },
    infrastructure::tmux_workspaces::tmux::{
        connection::TmuxConnectionInterface,
        tmux_format::{TmuxFilterAstBuilder, TmuxFilterNode},
        tmux_format_variables::{TmuxFormatField, TmuxFormatVariable},
    },
};

#[derive(Component)]
#[shaku(interface = TmuxPaneRepository)]
pub struct ImplPaneRepository {
    #[shaku(inject)]
    pub connection: Arc<dyn TmuxConnectionInterface>,
}

impl TmuxPaneRepository for ImplPaneRepository {
    fn get_panes(&self, filter: Option<TmuxFilterNode>, target: GetPanesTarget) -> Vec<TmuxPane> {
        let list_format = json!({
            "id": TmuxFormatVariable::PaneId.to_format(),
            "index": TmuxFormatVariable::PaneIndex.to_format(),
            "title": TmuxFormatVariable::PaneTitle.to_format(),
            "window_id": TmuxFormatVariable::WindowId.to_format(),
        })
        .to_string();
        let mut args = vec!["list-panes", "-F", &list_format];

        let target_args = match target {
            GetPanesTarget::Window { id: window_id } => vec!["-t", window_id],
            GetPanesTarget::Session { id: session_id } => {
                vec!["-s", "-t", session_id]
            }
            GetPanesTarget::None => vec![],
            GetPanesTarget::All => vec!["-a"],
        };

        let filter_str = filter.map(|x| x.as_string()).unwrap_or("".to_string());
        if !filter_str.is_empty() {
            args.extend(vec!["-f", &filter_str]);
        }

        args.extend(target_args);

        let res = self
            .connection
            .cmd(&args)
            .stderr_to_stdout()
            .read()
            .expect("Failed to get panes");
        res.lines()
            .map(|x| serde_json::from_str::<ListPaneResponse>(x).expect("Failed to get panes"))
            .map(|x| TmuxPane {
                id: x.id,
                index: x.index,
                title: x.title,
                window_id: x.window_id,
            })
            .collect()
    }

    fn kill_pane(&self, pane: Option<&TmuxPane>) {
        let mut args = vec!["kill-pane"];
        if let Some(pane_value) = pane {
            args.extend(["-t", &pane_value.id])
        }
        self.connection
            .cmd(&args)
            .stderr_to_stdout()
            .read()
            .expect("Failed to kill pane");
    }

    fn split_window(
        &self,
        pane: Option<&TmuxPane>,
        cwd: Option<&str>,
        direction: SplitDirection,
    ) -> TmuxPane {
        let mut args = vec!["split-window"];
        if let Some(cwd_path) = cwd {
            args.extend(["-c", cwd_path]);
        }

        match direction {
            SplitDirection::Horizontal => {
                args.push("-h");
            }
            SplitDirection::Vertical => {
                args.push("-v");
            }
        }

        args.extend(["-l", "50%"]);
        let old_panes: HashSet<String> = self
            .get_current_panes(pane)
            .iter()
            .map(|x| x.id.clone())
            .collect();

        let _ = self
            .connection
            .cmd(&args)
            .stderr_to_stdout()
            .read()
            .expect("Failed to get panes");

        let new_panes = self.get_current_panes(pane);
        new_panes
            .iter()
            .find(|x| !old_panes.contains(&x.id))
            .unwrap()
            .clone()
    }
}

impl ImplPaneRepository {
    /// Get panes for the window that contains the given pane, or all current panes if None.
    /// This avoids a circular dependency with TmuxWindowRepository by querying panes directly
    /// using a window_id filter instead of going through the window repository.
    fn get_current_panes(&self, pane: Option<&TmuxPane>) -> Vec<TmuxPane> {
        match pane {
            Some(pane_value) => self.get_panes(
                Some(TmuxFilterAstBuilder::build(|b| {
                    b.eq(
                        b.var(TmuxFormatVariable::WindowId),
                        b.const_val(&pane_value.window_id),
                    )
                })),
                GetPanesTarget::All,
            ),
            None => self.get_panes(None, GetPanesTarget::None),
        }
    }
}

#[derive(Deserialize)]
struct ListPaneResponse {
    id: String,
    index: String,
    title: String,
    window_id: String,
}
