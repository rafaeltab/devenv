use atty::Stream;
use inquire::{Confirm, Text};

use crate::{
    commands::command::RafaeltabCommand,
    domain::repositories::workspace::workspace_repository::WorkspaceRepository,
    utils::display::RafaeltabDisplay,
};

#[derive(Default)]
pub struct WorkspaceAddCommand;

pub struct WorkspaceAddOptions<'a> {
    pub display: &'a dyn RafaeltabDisplay,
    pub workspace_repository: &'a dyn WorkspaceRepository,
    pub interactive: Option<bool>,
    pub name: Option<String>,
    pub tags: Option<Vec<String>>,
    pub path: Option<String>,
}

impl<'a> RafaeltabCommand<WorkspaceAddOptions<'a>> for WorkspaceAddCommand {
    fn execute(&self, options: WorkspaceAddOptions) {
        let path = match options.path.clone() {
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

        let prompt_data = prompt_data(&options);

        // Build an id
        let id = prompt_data.name.to_lowercase().replace(' ', "_");

        let workspace = &options.workspace_repository.create_workspace(
            prompt_data.name,
            prompt_data.tags,
            path,
            id,
        );

        let _ = &options.display.display(workspace);
    }
}

fn prompt_data(options: &WorkspaceAddOptions) -> PromptData {
    let interactive = match options.interactive {
        Some(i) => i,
        None => atty::is(Stream::Stdout),
    };
    let name = match options.name.clone() {
        Some(n) => n,
        None => {
            if !interactive {
                panic!("Not interactive, but no name was provided.")
            }
            Text::new("Name:").prompt().expect("")
        }
    };

    let tags = match options.tags.clone() {
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
