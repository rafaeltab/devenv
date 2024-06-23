use std::collections::HashSet;

use duct::cmd;
use serde::Deserialize;
use serde_json::json;

use crate::{
    domain::{
        aggregates::tmux::{pane::TmuxPane, window::WindowIncludeFields},
        repositories::tmux::{
            pane_repository::{GetPanesTarget, SplitDirection, TmuxPaneRepository},
            window_repository::{GetWindowsTarget, TmuxWindowRepository},
        },
    },
    infrastructure::tmux::{
        tmux_format::{TmuxFilterAstBuilder, TmuxFilterNode},
        tmux_format_variables::{TmuxFormatField, TmuxFormatVariable},
    },
};

use super::tmux_client::TmuxRepository;

impl TmuxPaneRepository for TmuxRepository {
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

        let res = cmd("tmux", args)
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
        cmd("tmux", args)
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

        let _ = cmd("tmux", args)
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

impl TmuxRepository {
    fn get_current_panes(&self, pane: Option<&TmuxPane>) -> Vec<TmuxPane> {
        match pane {
            Some(pane_value) => {
                let window_id = &pane_value.window_id;
                let window = self.get_windows(
                    Some(TmuxFilterAstBuilder::build(|b| {
                        b.eq(b.var(TmuxFormatVariable::WindowId), b.const_val(window_id))
                    })),
                    WindowIncludeFields { panes: Some(()) },
                    GetWindowsTarget::None,
                );
                return window.first().unwrap().clone().panes.unwrap();
            }
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
