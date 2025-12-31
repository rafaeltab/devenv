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

    pub fn command(&self) -> Option<&str> {
        self.command.as_deref()
    }
}
