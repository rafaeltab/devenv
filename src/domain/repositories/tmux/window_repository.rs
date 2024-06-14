use crate::{
    domain::aggregates::tmux::{
        session::TmuxSession,
        window::{TmuxWindow, WindowIncludeFields},
    },
    infrastructure::tmux::tmux_format::TmuxFilterNode,
};

pub enum GetWindowsTarget<'a> {
    Session { id: &'a str },
    None,
    All,
}

pub trait TmuxWindowRepository {
    fn new_window(&self, new_widow: &NewWindowBuilder) -> TmuxWindow;
    fn delete_window(&self, session: Option<&TmuxWindow>);
    fn get_windows(
        &self,
        filter: Option<TmuxFilterNode>,
        include: WindowIncludeFields,
        target: GetWindowsTarget,
    ) -> Vec<TmuxWindow>;
}

#[allow(dead_code)]
pub struct NewWindowBuilder {
    pub dir: Option<String>,
    pub environment: Vec<(String, String)>,
    pub name: Option<String>,
    pub target: Option<TmuxSession>,
    pub command: Option<String>,
}

#[allow(dead_code)]
impl NewWindowBuilder {
    pub fn new() -> Self {
        NewWindowBuilder {
            dir: None,
            environment: vec![],
            name: None,
            target: None,
            command: None,
        }
    }

    pub fn with_dir(&self, dir: impl Into<String>) -> Self {
        NewWindowBuilder {
            dir: Some(dir.into()),
            environment: self.environment.clone(),
            name: self.name.clone(),
            target: self.target.clone(),
            command: self.command.clone(),
        }
    }

    pub fn add_env(&self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let mut environments = self.environment.clone();
        environments.push((key.into(), value.into()));
        NewWindowBuilder {
            dir: self.dir.clone(),
            environment: environments,
            name: self.name.clone(),
            target: self.target.clone(),
            command: self.command.clone(),
        }
    }

    pub fn with_name(&self, name: impl Into<String>) -> Self {
        NewWindowBuilder {
            dir: self.dir.clone(),
            environment: self.environment.clone(),
            name: Some(name.into()),
            target: self.target.clone(),
            command: self.command.clone(),
        }
    }

    pub fn with_target(&self, target: TmuxSession) -> Self {
        NewWindowBuilder {
            dir: self.dir.clone(),
            environment: self.environment.clone(),
            name: self.name.clone(),
            target: Some(target),
            command: self.command.clone(),
        }
    }

    pub fn with_command(&self, cmd: impl Into<String>) -> Self {
        NewWindowBuilder {
            dir: self.dir.clone(),
            environment: self.environment.clone(),
            name: self.name.clone(),
            target: self.target.clone(),
            command: Some(cmd.into()),
        }
    }
}
