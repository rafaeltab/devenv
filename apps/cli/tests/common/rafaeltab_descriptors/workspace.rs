use std::cell::RefCell;
use std::path::PathBuf;
use test_descriptors::{DirBuilder, GitBuilder};

// Thread-local storage for collecting workspaces during test setup
// This is automatically cleaned up after config creation or on panic
thread_local! {
    pub static WORKSPACES: RefCell<Vec<WorkspaceData>> = const { RefCell::new(Vec::new()) };
}

/// Panic guard to ensure WORKSPACES is cleaned up even if test panics
struct WorkspacePanicGuard;

impl Drop for WorkspacePanicGuard {
    fn drop(&mut self) {
        if std::thread::panicking() {
            WORKSPACES.with(|w| w.borrow_mut().clear());
        }
    }
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

/// Builder for configuring a rafaeltab workspace.
///
/// This builder allows you to configure workspace metadata including tags
/// and worktree settings. The workspace will be registered in the global
/// config when the config descriptor is created.
///
/// # Example
/// ```ignore
/// use test_descriptors::TestEnvironment;
///
/// let env = TestEnvironment::describe(|root| {
///     root.rafaeltab_config(|_c| {});
///     root.test_dir(|td| {
///         td.dir("my-project", |d| {
///             d.rafaeltab_workspace("my_project", "My Project", |w| {
///                 w.tag("rust");
///                 w.tag("cli");
///             });
///         });
///     });
/// }).create();
/// ```
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
        // Ensure cleanup even on panic
        let _guard = WorkspacePanicGuard;

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

/// Mixin trait for `DirBuilder` to add workspace registration capability.
///
/// This trait allows you to register a rafaeltab workspace at a directory's path.
/// The directory itself is created by the `DirBuilder`, and this mixin just
/// registers the workspace metadata in the global config.
///
/// Use this when you want to create a workspace that points to a simple directory.
///
/// # Example
/// ```ignore
/// use test_descriptors::TestEnvironment;
///
/// let env = TestEnvironment::describe(|root| {
///     root.rafaeltab_config(|_c| {});
///     root.test_dir(|td| {
///         td.dir("my-workspace", |d| {
///             // Register this directory as a workspace
///             d.rafaeltab_workspace("my_ws", "My Workspace", |w| {
///                 w.tag("rust");
///             });
///         });
///     });
/// }).create();
/// ```
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

/// Mixin trait for `GitBuilder` to add workspace registration capability.
///
/// This trait allows you to register a rafaeltab workspace at a git repository's path.
/// The git repository itself is created by the `GitBuilder`, and this mixin just
/// registers the workspace metadata in the global config.
///
/// Use this when you want to create a workspace that points to a git repository.
///
/// # Example
/// ```ignore
/// use test_descriptors::TestEnvironment;
///
/// let env = TestEnvironment::describe(|root| {
///     root.rafaeltab_config(|_c| {});
///     root.test_dir(|td| {
///         td.dir("projects", |d| {
///             d.git("my-repo", |g| {
///                 g.branch("main", |b| {
///                     b.commit("Initial commit", |c| {
///                         c.file("README.md", "# My Project");
///                     });
///                 });
///                 // Register this git repo as a workspace
///                 g.rafaeltab_workspace("my_repo", "My Repo", |w| {
///                     w.tag("javascript");
///                 });
///             });
///         });
///     });
/// }).create();
/// ```
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
