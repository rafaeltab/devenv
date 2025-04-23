use std::collections::HashSet;

use duct::cmd;
use serde::Deserialize;
use serde_json::json;

use crate::{
    domain::{
        aggregates::tmux::window::{TmuxWindow, WindowIncludeFields},
        repositories::tmux::{
            pane_repository::{GetPanesTarget, TmuxPaneRepository},
            window_repository::{GetWindowsTarget, NewWindowBuilder, TmuxWindowRepository},
        },
    },
    infrastructure::tmux::{
        tmux_format::{TmuxFilterAstBuilder, TmuxFilterNode},
        tmux_format_variables::{TmuxFormatField, TmuxFormatVariable},
    }, storage::tmux::TmuxStorage,
};

use super::tmux_client::TmuxRepository;

impl<'a, TTmuxStorage: TmuxStorage> TmuxWindowRepository for TmuxRepository<'a, TTmuxStorage> {
    fn new_window(&self, new_window: &NewWindowBuilder) -> TmuxWindow {
        let mut args = vec!["new-window"];
        if let Some(dir_val) = &new_window.dir {
            args.extend(["-c", &dir_val]);
        }
        let env: Vec<String> = new_window
            .environment
            .iter()
            .map(|(key, value)| format!("{}={}", &key, &value))
            .collect();

        for val in &env {
            args.extend(["-e", val]);
        }

        if let Some(name) = &new_window.name {
            args.extend(["-n", name]);
        }

        if let Some(target) = &new_window.target {
            args.extend(["-t", &target.id]);
        }

        let window_command_with_shell = command_with_shell(new_window.command.clone());

        if let Some(ref command) = window_command_with_shell {
            args.push(command);
        }

        let list_format = json!({
            "id": TmuxFormatVariable::WindowId.to_format(),
            "index": TmuxFormatVariable::WindowIndex.to_format(),
            "name": TmuxFormatVariable::WindowName.to_format(),
            "session_id": TmuxFormatVariable::SessionId.to_format(),
        })
        .to_string();
        args.extend(["-P", "-F", &list_format]);
        let out = cmd("tmux", args)
            .stderr_to_stdout()
            .read()
            .expect("Failed to create window");

        let response = serde_json::from_str::<ListWindowsResponse>(&out)
            .expect("Failed to parse window response");
        let panes = self.get_panes(
            Some(TmuxFilterAstBuilder::build(|b| {
                b.eq(
                    b.var(TmuxFormatVariable::WindowId),
                    b.const_val(&response.id),
                )
            })),
            GetPanesTarget::All,
        );

        TmuxWindow {
            id: response.id,
            index: response.index,
            name: response.name,
            panes: Some(panes),
            include_fields: WindowIncludeFields { panes: Some(()) },
        }
    }

    fn delete_window(&self, window: Option<&TmuxWindow>) {
        let mut args = vec!["kill-window"];
        if let Some(wind) = window {
            args.extend(["-t", &wind.id]);
        }
        cmd("tmux", args)
            .stderr_to_stdout()
            .read()
            .expect("Failed to kill window");
    }

    fn get_windows(
        &self,
        filter: Option<TmuxFilterNode>,
        include: WindowIncludeFields,
        target: GetWindowsTarget,
    ) -> Vec<TmuxWindow> {
        let list_format = json!({
            "id": TmuxFormatVariable::WindowId.to_format(),
            "index": TmuxFormatVariable::WindowIndex.to_format(),
            "name": TmuxFormatVariable::WindowName.to_format(),
            "session_id": TmuxFormatVariable::SessionId.to_format(),
        })
        .to_string();
        let mut args = vec!["list-windows", "-F", &list_format];

        let target_filter = match target {
            GetWindowsTarget::Session { id } => vec!["-t", id],
            GetWindowsTarget::None => vec![],
            GetWindowsTarget::All => vec!["-a"],
        };
        args.extend(target_filter);

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
            .expect("Failed to get windows");
        let responses: Vec<ListWindowsResponse> = res
            .lines()
            .map(|x| serde_json::from_str::<ListWindowsResponse>(x).expect("Failed to get windows"))
            .collect();

        let window_ids: HashSet<String> = responses.iter().map(|x| x.id.clone()).collect();
        match include.panes {
            Some(_) => {
                let panes = self.get_panes(
                    Some(TmuxFilterAstBuilder::build(|b| {
                        b.any(
                            window_ids
                                .iter()
                                .map(|x| b.eq(b.const_val(x), b.var(TmuxFormatVariable::WindowId)))
                                .collect(),
                        )
                    })),
                    GetPanesTarget::All,
                );
                responses
                    .iter()
                    .map(|x| TmuxWindow {
                        name: x.name.clone(),
                        id: x.id.clone(),
                        index: x.index.clone(),
                        panes: Some(
                            panes
                                .clone()
                                .iter()
                                .filter(|y| y.window_id == x.id)
                                .cloned()
                                .collect(),
                        ),
                        include_fields: include.clone(),
                    })
                    .collect()
            }
            None => responses
                .iter()
                .map(|x| TmuxWindow {
                    name: x.name.clone(),
                    id: x.id.clone(),
                    index: x.index.clone(),
                    panes: None,
                    include_fields: include.clone(),
                })
                .collect(),
        }
    }
}

fn command_with_shell(cmd: Option<String>) -> Option<String> {
    cmd.map(|cmd_str| cmd_str + "; exec $SHELL")
}

#[derive(Deserialize)]
struct ListWindowsResponse {
    name: String,
    id: String,
    index: String,
}
