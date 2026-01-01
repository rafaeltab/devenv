use std::cell::RefCell;
use std::path::PathBuf;
use test_descriptors::{DirBuilder, GitBuilder};

// Thread-local storage for collecting workspaces during test setup
thread_local! {
    pub static WORKSPACES: RefCell<Vec<WorkspaceData>> = RefCell::new(Vec::new());
}

/// Internal data structure for storing workspace configuration
#[derive(Debug, Clone)]
pub struct WorkspaceData {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub tags: Vec<String>,
    pub worktree: Option<WorktreeConfig>,
}

/// Worktree configuration for a workspace
#[derive(Debug, Clone)]
pub struct WorktreeConfig {
    pub on_create: Vec<String>,
    pub symlink_files: Vec<String>,
}

/// Builder for configuring a workspace
pub struct WorkspaceBuilder {
    id: String,
    name: String,
    path: PathBuf,
    tags: Vec<String>,
    worktree: Option<WorktreeConfig>,
}

impl WorkspaceBuilder {
    pub(crate) fn new(id: &str, name: &str, path: PathBuf) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            path,
            tags: Vec::new(),
            worktree: None,
        }
    }

    /// Add a tag to this workspace
    pub fn tag(&mut self, tag: &str) {
        self.tags.push(tag.to_string());
    }

    /// Configure worktree settings for this workspace
    pub fn worktree(&mut self, on_create: &[&str], symlink_files: &[&str]) {
        self.worktree = Some(WorktreeConfig {
            on_create: on_create.iter().map(|s| s.to_string()).collect(),
            symlink_files: symlink_files.iter().map(|s| s.to_string()).collect(),
        });
    }

    /// Register the workspace data (called internally, doesn't create directory)
    pub(crate) fn register(&self) {
        WORKSPACES.with(|workspaces| {
            workspaces.borrow_mut().push(WorkspaceData {
                id: self.id.clone(),
                name: self.name.clone(),
                path: self.path.clone(),
                tags: self.tags.clone(),
                worktree: self.worktree.clone(),
            });
        });
    }
}

/// Descriptor for creating a workspace (standalone, creates directory)
///
/// Note: This is kept for backwards compatibility but the preferred way
/// is to use `RafaeltabDirMixin` or `RafaeltabGitMixin`.
#[derive(Debug)]
pub struct WorkspaceDescriptor {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) tags: Vec<String>,
    pub(crate) worktree: Option<WorktreeConfig>,
}

/// Mixin trait for DirBuilder - registers workspace at directory path
///
/// The directory is created by the DirBuilder, so this just registers
/// the workspace in the config at the directory's path.
pub trait RafaeltabDirMixin {
    fn rafaeltab_workspace<F>(&mut self, id: &str, name: &str, f: F)
    where
        F: FnOnce(&mut WorkspaceBuilder);
}

impl RafaeltabDirMixin for DirBuilder {
    fn rafaeltab_workspace<F>(&mut self, id: &str, name: &str, f: F)
    where
        F: FnOnce(&mut WorkspaceBuilder),
    {
        // The workspace path is the directory's path (our path)
        let our_path = self.our_path();
        let mut builder = WorkspaceBuilder::new(id, name, our_path);
        f(&mut builder);
        // Register the workspace (directory is already created by DirBuilder)
        builder.register();
    }
}

/// Mixin trait for GitBuilder - registers workspace at git repo path
///
/// The git repository is created by the GitBuilder, so this just registers
/// the workspace in the config at the git repo's path.
pub trait RafaeltabGitMixin {
    fn rafaeltab_workspace<F>(&mut self, id: &str, name: &str, f: F)
    where
        F: FnOnce(&mut WorkspaceBuilder);
}

impl RafaeltabGitMixin for GitBuilder {
    fn rafaeltab_workspace<F>(&mut self, id: &str, name: &str, f: F)
    where
        F: FnOnce(&mut WorkspaceBuilder),
    {
        // The workspace path is the git repo's path
        let repo_path = self.repo_path();
        let mut builder = WorkspaceBuilder::new(id, name, repo_path);
        f(&mut builder);
        // Register the workspace (git repo is created by GitBuilder)
        builder.register();
    }
}
