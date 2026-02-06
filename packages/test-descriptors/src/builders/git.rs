use super::changes::{StagedBuilder, StagedChanges, UnstagedBuilder, UnstagedChanges};
use super::tmux::SessionBuilder;
use crate::builders::tmux::HierarchicalTmuxSessionDescriptor;
use crate::descriptor::{
    BranchDescriptor, CommitDescriptor, CreateContext, CreateError, Descriptor, GitRepoDescriptor,
    RemoteDescriptor,
};
use std::path::PathBuf;

pub struct GitBuilder {
    name: String,
    parent_path: PathBuf,
    branches: Vec<BranchDescriptor>,
    remotes: Vec<RemoteDescriptor>,
    initial_branch: Option<String>,
    staged: Option<StagedChanges>,
    unstaged: Option<UnstagedChanges>,
    tmux_sessions: Vec<HierarchicalTmuxSessionDescriptor>,
}

impl GitBuilder {
    pub(crate) fn new(name: &str, parent_path: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            parent_path,
            branches: Vec::new(),
            remotes: Vec::new(),
            initial_branch: None,
            staged: None,
            unstaged: None,
            tmux_sessions: Vec::new(),
        }
    }

    /// Get the full path this git repository will be created at
    pub fn repo_path(&self) -> PathBuf {
        self.parent_path.join(&self.name)
    }

    pub fn branch<F>(&mut self, name: &str, f: F)
    where
        F: FnOnce(&mut BranchBuilder),
    {
        let mut builder = BranchBuilder::new(name);
        f(&mut builder);
        self.branches.push(builder.build());
    }

    pub fn branch_from<F>(&mut self, name: &str, base: &str, f: F)
    where
        F: FnOnce(&mut BranchBuilder),
    {
        let mut builder = BranchBuilder::from(name, base);
        f(&mut builder);
        self.branches.push(builder.build());
    }

    pub fn remote(&mut self, name: &str) {
        self.remotes.push(RemoteDescriptor::new(name));
    }

    pub fn initial_branch(&mut self, name: &str) {
        self.initial_branch = Some(name.to_string());
    }

    /// Add staged (but not committed) changes to this repository
    pub fn staged<F>(&mut self, f: F)
    where
        F: FnOnce(&mut StagedBuilder),
    {
        let mut builder = StagedBuilder::new();
        f(&mut builder);
        self.staged = Some(builder.build());
    }

    /// Add unstaged changes to this repository
    pub fn unstaged<F>(&mut self, f: F)
    where
        F: FnOnce(&mut UnstagedBuilder),
    {
        let mut builder = UnstagedBuilder::new();
        f(&mut builder);
        self.unstaged = Some(builder.build());
    }

    /// Create a tmux session with working directory set to this repository
    pub fn tmux_session<F>(&mut self, name: &str, f: F)
    where
        F: FnOnce(&mut SessionBuilder),
    {
        let mut builder = SessionBuilder::new(name, self.repo_path());
        f(&mut builder);
        self.tmux_sessions.push(builder.build());
    }

    pub(crate) fn build(self) -> HierarchicalGitRepoDescriptor {
        HierarchicalGitRepoDescriptor {
            name: self.name,
            parent_path: self.parent_path,
            branches: self.branches,
            remotes: self.remotes,
            initial_branch: self.initial_branch.unwrap_or_else(|| "main".to_string()),
            staged: self.staged,
            unstaged: self.unstaged,
            tmux_sessions: self.tmux_sessions,
        }
    }
}

pub struct BranchBuilder {
    name: String,
    base: Option<String>,
    commits: Vec<CommitDescriptor>,
}

impl BranchBuilder {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            base: None,
            commits: Vec::new(),
        }
    }

    pub(crate) fn from(name: &str, base: &str) -> Self {
        Self {
            name: name.to_string(),
            base: Some(base.to_string()),
            commits: Vec::new(),
        }
    }

    pub fn commit<F>(&mut self, message: &str, f: F)
    where
        F: FnOnce(&mut CommitBuilder),
    {
        let mut builder = CommitBuilder::new(message);
        f(&mut builder);
        self.commits.push(builder.build());
    }

    pub(crate) fn build(self) -> BranchDescriptor {
        let mut branch = if let Some(base) = self.base {
            BranchDescriptor::from(&self.name, &base)
        } else {
            BranchDescriptor::new(&self.name)
        };

        for commit in self.commits {
            branch = branch.with_commit(commit);
        }

        branch
    }
}

pub struct CommitBuilder {
    message: String,
    files: Vec<(String, String)>,
    deletes: Vec<String>,
    pushed_to: Option<String>,
    pushed_as: Option<String>,
}

impl CommitBuilder {
    pub(crate) fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            files: Vec::new(),
            deletes: Vec::new(),
            pushed_to: None,
            pushed_as: None,
        }
    }

    pub fn file(&mut self, path: &str, content: &str) {
        self.files.push((path.to_string(), content.to_string()));
    }

    pub fn delete(&mut self, path: &str) {
        self.deletes.push(path.to_string());
    }

    pub fn pushed(&mut self, remote: &str) {
        self.pushed_to = Some(remote.to_string());
    }

    pub fn pushed_as(&mut self, remote: &str, branch: &str) {
        self.pushed_to = Some(remote.to_string());
        self.pushed_as = Some(branch.to_string());
    }

    pub(crate) fn build(self) -> CommitDescriptor {
        let mut commit = CommitDescriptor::new(&self.message);

        for (path, content) in self.files {
            commit = commit.with_file(&path, &content);
        }

        for path in self.deletes {
            commit = commit.with_delete(&path);
        }

        if let Some(remote) = self.pushed_to {
            if let Some(branch) = self.pushed_as {
                commit = commit.pushed_as(&remote, &branch);
            } else {
                commit = commit.pushed_to(&remote);
            }
        }

        commit
    }
}

/// Hierarchical Git repo descriptor that knows its parent path
#[derive(Debug)]
pub struct HierarchicalGitRepoDescriptor {
    name: String,
    parent_path: PathBuf,
    branches: Vec<BranchDescriptor>,
    remotes: Vec<RemoteDescriptor>,
    initial_branch: String,
    staged: Option<StagedChanges>,
    unstaged: Option<UnstagedChanges>,
    tmux_sessions: Vec<HierarchicalTmuxSessionDescriptor>,
}

impl Descriptor for HierarchicalGitRepoDescriptor {
    fn create(&self, context: &CreateContext) -> Result<(), CreateError> {
        let path = self.parent_path.join(&self.name);

        // Build the underlying GitRepoDescriptor
        let mut repo = GitRepoDescriptor::new(&self.name).with_initial_branch(&self.initial_branch);

        for remote in &self.remotes {
            repo = repo.with_remote(remote.clone());
        }

        for branch in &self.branches {
            repo = repo.with_branch(branch.clone());
        }

        // Override the path by creating our own context view
        // We need to actually create at the right path
        // The GitRepoDescriptor uses context.root_path().join(name)
        // but we need parent_path.join(name)

        // Create parent directories if needed
        std::fs::create_dir_all(&self.parent_path)?;

        // Create a sub-context rooted at parent_path
        let sub_context = CreateContext::new(self.parent_path.clone());

        // Copy over the tmux socket if set
        if let Some(socket) = context.tmux_socket() {
            sub_context.set_tmux_socket(socket);
        }

        // Create the repo using sub-context
        repo.create(&sub_context)?;

        // Apply staged changes (after all commits, on initial branch)
        if let Some(staged) = &self.staged {
            staged.apply(&path)?;
        }

        // Apply unstaged changes (after staged, so they remain unstaged)
        if let Some(unstaged) = &self.unstaged {
            unstaged.apply(&path)?;
        }

        // Register in the original context's registry
        context
            .registry()
            .borrow_mut()
            .register_git_repo(self.name.clone(), path);

        // Create tmux sessions (working dir will be the repo path)
        for session in &self.tmux_sessions {
            session.create(context)?;
        }

        Ok(())
    }
}
