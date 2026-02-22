use std::sync::Arc;

use atty::Stream;
use inquire::{Confirm, Text};
use shaku::{Component, Interface};

use crate::{
    domain::tmux_workspaces::repositories::workspace::workspace_repository::WorkspaceRepository,
    utils::display::RafaeltabDisplay,
};

pub trait WorkspaceAddCommandInterface: Interface {
    fn execute(&self, args: WorkspaceAddArgs);
}

pub struct WorkspaceAddArgs<'a> {
    pub display: &'a dyn RafaeltabDisplay,
    pub interactive: Option<bool>,
    pub name: Option<String>,
    pub tags: Option<Vec<String>>,
    pub path: Option<String>,
}

#[derive(Component)]
#[shaku(interface = WorkspaceAddCommandInterface)]
pub struct WorkspaceAddCommand {
    #[shaku(inject)]
    workspace_repository: Arc<dyn WorkspaceRepository>,
}

impl WorkspaceAddCommandInterface for WorkspaceAddCommand {
    fn execute(&self, args: WorkspaceAddArgs) {
        let path = match args.path.clone() {
            Some(path) => path,
            None => {
                let curr_dir_err_msg = "Unable to get the current directory";
                std::env::current_dir()
                    .expect(curr_dir_err_msg)
                    .to_str()
                    .expect(curr_dir_err_msg)
                    .to_string()
            }
        };

        let prompt_data = prompt_data(&args);

        // Build an id
        let id = prompt_data.name.to_lowercase().replace(' ', "_");

        let workspace = &self.workspace_repository.create_workspace(
            prompt_data.name,
            prompt_data.tags,
            path,
            id,
        );

        args.display.display(workspace);
    }
}

fn prompt_data(args: &WorkspaceAddArgs) -> PromptData {
    let interactive = match args.interactive {
        Some(i) => i,
        None => atty::is(Stream::Stdout),
    };
    let name = match args.name.clone() {
        Some(n) => n,
        None => {
            if !interactive {
                panic!("Not interactive, but no name was provided.")
            }
            Text::new("Name:").prompt().expect("")
        }
    };

    let tags = match args.tags.clone() {
        Some(t) => t,
        None => prompt_tags(interactive),
    };

    PromptData { name, tags }
}

fn prompt_tags(interactive: bool) -> Vec<String> {
    if !interactive {
        return vec![];
    }
    loop {
        let tags = Text::new("Tags (comma separated):").prompt().expect("");
        if tags.trim().is_empty() {
            let confirmation = Confirm::new("You entered no tags, is that correct?")
                .prompt()
                .expect("");

            if confirmation {
                return vec![];
            }
        }

        let tag_list: Vec<String> = tags.split(',').map(|x| x.trim().to_string()).collect();

        let tag_presentation = tag_list
            .iter()
            .map(|x| format!("- {}", x))
            .collect::<Vec<String>>()
            .join("\n");

        println!("{}", tag_presentation);

        let confirmation = Confirm::new("Is the list above correct?")
            .prompt()
            .expect("");

        if confirmation {
            return tag_list;
        }
    }
}

struct PromptData {
    pub name: String,
    pub tags: Vec<String>,
}
