use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct TmuxSessionInfo {
    pub name: String,
    pub working_dir: PathBuf,
}

#[derive(Debug, Default)]
pub struct ResourceRegistry {
    git_repos: HashMap<String, PathBuf>,
    worktrees: HashMap<(String, String), PathBuf>,
    tmux_sessions: HashMap<String, TmuxSessionInfo>,
    dirs: HashMap<String, PathBuf>,
}

impl ResourceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_git_repo(&mut self, name: String, path: PathBuf) {
        self.git_repos.insert(name, path);
    }

    pub fn get_git_repo(&self, name: &str) -> Option<&PathBuf> {
        self.git_repos.get(name)
    }

    pub fn register_worktree(&mut self, repo: String, branch: String, path: PathBuf) {
        self.worktrees.insert((repo, branch), path);
    }

    pub fn get_worktree(&self, repo: &str, branch: &str) -> Option<&PathBuf> {
        self.worktrees.get(&(repo.to_string(), branch.to_string()))
    }

    pub fn register_tmux_session(&mut self, name: String, info: TmuxSessionInfo) {
        self.tmux_sessions.insert(name, info);
    }

    pub fn get_tmux_session(&self, name: &str) -> Option<&TmuxSessionInfo> {
        self.tmux_sessions.get(name)
    }

    pub fn register_dir(&mut self, name: String, path: PathBuf) {
        self.dirs.insert(name, path);
    }

    pub fn get_dir(&self, name: &str) -> Option<&PathBuf> {
        self.dirs.get(name)
    }
}
