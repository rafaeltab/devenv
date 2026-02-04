use crate::builders::RootBuilder;
use crate::descriptor::{CreateContext, Descriptor, TmuxClientHandle, TmuxSocket};
use crate::queries::{DirRef, GitRepoRef, TmuxSessionRef, WorktreeRef};
use crate::testers::TesterFactory;
use std::path::Path;
use tempfile::TempDir;

pub struct TestEnvironment {
    temp_dir: TempDir,
    context: CreateContext,
    tmux_socket: TmuxSocket,
    descriptors: Vec<Box<dyn Descriptor>>,
    created: bool,
    tmux_client: Option<TmuxClientHandle>,
}

impl TestEnvironment {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let root_path = temp_dir.path().to_path_buf();
        let context = CreateContext::new(root_path);
        let tmux_socket = TmuxSocket::new();

        // Set the tmux socket in the context
        context.set_tmux_socket(tmux_socket.name().to_string());

        Self {
            temp_dir,
            context,
            tmux_socket,
            descriptors: Vec::new(),
            created: false,
            tmux_client: None,
        }
    }

    pub fn add_descriptor<D: Descriptor + 'static>(&mut self, descriptor: D) {
        self.descriptors.push(Box::new(descriptor));
    }

    pub(crate) fn add_boxed_descriptor(&mut self, descriptor: Box<dyn Descriptor>) {
        self.descriptors.push(descriptor);
    }

    pub fn create(mut self) -> Self {
        if self.created {
            return self;
        }

        for descriptor in &self.descriptors {
            descriptor
                .create(&self.context)
                .expect("Failed to create descriptor");
        }

        // Create the tmux client if one was configured
        if let Some(client_descriptor) = self.context.take_pending_client() {
            let socket_name = self.tmux_socket.name().to_string();
            let client = client_descriptor
                .create_client(&socket_name)
                .expect("Failed to create tmux client");
            self.tmux_client = Some(client);
        }

        self.created = true;
        self
    }

    pub fn root_path(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn tmux_socket(&self) -> &str {
        self.tmux_socket.name()
    }

    pub fn tmux(&self) -> &TmuxSocket {
        &self.tmux_socket
    }

    pub fn context(&self) -> &CreateContext {
        &self.context
    }

    /// Create a test environment using the hierarchical builder API
    pub fn describe<F>(f: F) -> Self
    where
        F: FnOnce(&mut RootBuilder),
    {
        let mut env = Self::new();
        {
            let mut root = RootBuilder::new(&mut env);
            f(&mut root);
        }
        env
    }

    /// Find a directory by name (query API)
    pub fn find_dir(&self, name: &str) -> Option<DirRef<'_>> {
        self.context
            .registry()
            .borrow()
            .get_dir(name)
            .map(|path| DirRef {
                path: path.clone(),
                _env: self,
            })
    }

    /// Find a git repository by name (query API)
    pub fn find_git_repo(&self, name: &str) -> Option<GitRepoRef<'_>> {
        self.context
            .registry()
            .borrow()
            .get_git_repo(name)
            .map(|path| GitRepoRef {
                name: name.to_string(),
                path: path.clone(),
                _env: self,
            })
    }

    /// Find a tmux session by name (query API)
    pub fn find_tmux_session(&self, name: &str) -> Option<TmuxSessionRef<'_>> {
        self.context
            .registry()
            .borrow()
            .get_tmux_session(name)
            .map(|info| TmuxSessionRef {
                name: info.name.clone(),
                working_dir: info.working_dir.clone(),
                env: self,
            })
    }

    /// Find a git worktree by repository name and branch (query API)
    pub fn find_worktree(&self, repo_name: &str, branch: &str) -> Option<WorktreeRef<'_>> {
        self.context
            .registry()
            .borrow()
            .get_worktree(repo_name, branch)
            .map(|path| WorktreeRef {
                repo_name: repo_name.to_string(),
                branch: branch.to_string(),
                path: path.clone(),
                _env: self,
            })
    }

    /// Get a factory for creating testers.
    pub fn testers(&self) -> TesterFactory<'_> {
        TesterFactory::new(self)
    }

    /// Check if this environment has a tmux client configured.
    pub fn has_tmux_client(&self) -> bool {
        self.tmux_client.is_some()
    }

    /// Get the tmux client handle if one is configured.
    pub fn tmux_client(&self) -> Option<&TmuxClientHandle> {
        self.tmux_client.as_ref()
    }

    /// Set the tmux client handle (called during environment creation).
    #[allow(dead_code)]
    pub(crate) fn set_tmux_client(&mut self, client: TmuxClientHandle) {
        self.tmux_client = Some(client);
    }
}

impl Default for TestEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        // Kill tmux server to clean up all sessions
        let _ = self.tmux_socket.kill_server();

        // TempDir will automatically clean up when dropped
    }
}
