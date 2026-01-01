#[derive(Debug, Clone)]
pub struct WindowDescriptor {
    name: String,
    command: Option<String>,
}

impl WindowDescriptor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            command: None,
        }
    }

    pub fn with_command(mut self, cmd: &str) -> Self {
        self.command = Some(cmd.to_string());
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn command(&self) -> Option<WindowCommand<'_>> {
        self.command.as_deref().map(WindowCommand)
    }
}

/// A wrapper around a window command that provides fluent transformation methods.
#[derive(Debug, Clone, Copy)]
pub struct WindowCommand<'a>(pub &'a str);

impl<'a> WindowCommand<'a> {
    /// Wraps the command to keep the window alive after execution.
    ///
    /// When a tmux window is created with a command, the window closes when
    /// the command exits. This method wraps the command with `; exec $SHELL`
    /// so a shell is spawned after the command completes, keeping the window alive.
    pub fn with_persistent_shell(self) -> String {
        format!("{}; exec $SHELL", self.0)
    }
}
