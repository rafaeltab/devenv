use shaku::{Component, Interface};
use std::env;

/// Trait for config path configuration
pub trait ConfigPathProvider: Interface {
    fn path(&self) -> &str;
}

/// Configuration path option - wraps the path to the config file
#[derive(Component, Clone, Debug)]
#[shaku(interface = ConfigPathProvider)]
pub struct ConfigPathOption {
    pub path: String,
}

impl ConfigPathProvider for ConfigPathOption {
    fn path(&self) -> &str {
        &self.path
    }
}

impl ConfigPathOption {
    /// Create from a resolved path (for testing)
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

/// Trait for socket name configuration
pub trait SocketNameProvider: Interface {
    fn socket_name(&self) -> Option<String>;
}

/// Socket name option - reads from RAFAELTAB_TMUX_SOCKET env var every time
/// This ensures test isolation works correctly - tests set the env var before running
#[derive(Component, Clone, Debug)]
#[shaku(interface = SocketNameProvider)]
pub struct SocketNameOption {
    #[shaku(default)]
    _marker: (),
}

impl SocketNameProvider for SocketNameOption {
    fn socket_name(&self) -> Option<String> {
        // Read from environment variable every time - ensures test isolation
        env::var("RAFAELTAB_TMUX_SOCKET").ok()
    }
}

impl SocketNameOption {
    /// Create new instance (reads from env on each access)
    pub fn new() -> Self {
        Self { _marker: () }
    }
}

impl Default for SocketNameOption {
    fn default() -> Self {
        Self::new()
    }
}
