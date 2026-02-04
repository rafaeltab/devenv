use super::registry::ResourceRegistry;
use super::tmux_client::TmuxClientDescriptor;
use std::cell::RefCell;
use std::path::PathBuf;

pub struct CreateContext {
    root_path: PathBuf,
    registry: RefCell<ResourceRegistry>,
    tmux_socket: RefCell<Option<String>>,
    config_path: RefCell<Option<PathBuf>>,
    /// Pending client descriptor to be created after all other descriptors.
    pending_client: RefCell<Option<TmuxClientDescriptor>>,
}

impl CreateContext {
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            root_path,
            registry: RefCell::new(ResourceRegistry::new()),
            tmux_socket: RefCell::new(None),
            config_path: RefCell::new(None),
            pending_client: RefCell::new(None),
        }
    }

    pub fn root_path(&self) -> &PathBuf {
        &self.root_path
    }

    pub fn register_resource(&self, key: String, path: PathBuf) {
        self.registry.borrow_mut().register_dir(key, path);
    }

    pub fn get_resource(&self, key: &str) -> Option<PathBuf> {
        self.registry.borrow().get_dir(key).cloned()
    }

    pub fn registry(&self) -> &RefCell<ResourceRegistry> {
        &self.registry
    }

    pub fn set_tmux_socket(&self, socket: String) {
        *self.tmux_socket.borrow_mut() = Some(socket);
    }

    pub fn tmux_socket(&self) -> Option<String> {
        self.tmux_socket.borrow().clone()
    }

    pub fn set_config_path(&self, path: PathBuf) {
        *self.config_path.borrow_mut() = Some(path);
    }

    pub fn config_path(&self) -> Option<PathBuf> {
        self.config_path.borrow().clone()
    }

    /// Register a pending client descriptor to be created after the session.
    pub fn set_pending_client(&self, client: TmuxClientDescriptor) {
        *self.pending_client.borrow_mut() = Some(client);
    }

    /// Take the pending client descriptor, if any.
    pub fn take_pending_client(&self) -> Option<TmuxClientDescriptor> {
        self.pending_client.borrow_mut().take()
    }

    /// Check if there is a pending client.
    pub fn has_pending_client(&self) -> bool {
        self.pending_client.borrow().is_some()
    }
}
