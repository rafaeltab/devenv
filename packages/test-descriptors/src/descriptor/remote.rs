use super::context::CreateContext;
use super::error::CreateError;
use std::path::PathBuf;
use std::process::Command;
use uuid::Uuid;

struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run_git(args: &[&str], dir: &PathBuf) -> Result<(), CreateError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CreateError::GitError(format!(
            "Git command {:?} failed: {}",
            args, stderr
        )));
    }
    Ok(())
}

fn validate_branch_name(name: &str) -> Result<(), CreateError> {
    if name.contains("..")
        || name.contains('~')
        || name.contains('^')
        || name.contains(':')
        || name.starts_with('-')
    {
        return Err(CreateError::GitError(format!(
            "Invalid branch name '{}': contains invalid characters",
            name
        )));
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub struct RemoteDescriptor {
    name: String,
    branches: Vec<RemoteBranchDescriptor>,
}

impl RemoteDescriptor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            branches: Vec::new(),
        }
    }

    pub fn with_branch(mut self, branch: RemoteBranchDescriptor) -> Self {
        self.branches.push(branch);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn branches(&self) -> &[RemoteBranchDescriptor] {
        &self.branches
    }

    pub fn create_bare_repo(&self, context: &CreateContext) -> Result<PathBuf, CreateError> {
        let bare_repo_name = format!("bare-{}-{}.git", self.name, Uuid::new_v4());
        let bare_path = context.root_path().join(&bare_repo_name);

        std::fs::create_dir_all(&bare_path)?;

        let output = Command::new("git")
            .args(["init", "--bare"])
            .current_dir(&bare_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CreateError::GitError(format!(
                "Failed to init bare repo: {}",
                stderr
            )));
        }

        run_git(&["config", "user.name", "Test User"], &bare_path)?;
        run_git(&["config", "user.email", "test@example.com"], &bare_path)?;

        if !self.branches.is_empty() {
            self.create_branches_in_bare_repo(&bare_path)?;
        }

        Ok(bare_path)
    }

    fn create_branches_in_bare_repo(&self, bare_path: &PathBuf) -> Result<(), CreateError> {
        let temp_work_dir = bare_path.parent().unwrap().join(format!("temp-work-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&temp_work_dir)?;
        let _guard = TempDirGuard(temp_work_dir.clone());

        run_git(&["init"], &temp_work_dir)?;
        run_git(&["config", "user.name", "Test User"], &temp_work_dir)?;
        run_git(&["config", "user.email", "test@example.com"], &temp_work_dir)?;

        std::fs::write(temp_work_dir.join("README.md"), "# Remote\n")?;
        run_git(&["add", "README.md"], &temp_work_dir)?;
        run_git(&["commit", "-m", "Initial commit"], &temp_work_dir)?;

        run_git(
            &["remote", "add", "origin", bare_path.to_string_lossy().as_ref()],
            &temp_work_dir,
        )?;
        run_git(&["push", "origin", "main:main"], &temp_work_dir)?;

        let mut pushed_branches = std::collections::HashSet::new();
        pushed_branches.insert("main".to_string());

        let mut branches_to_create: Vec<_> = self.branches.iter().collect();
        let mut created_order = Vec::new();

        while !branches_to_create.is_empty() {
            let mut progress = false;
            let mut remaining = Vec::new();

            for branch in branches_to_create {
                if let Some(base) = &branch.base {
                    if !pushed_branches.contains(base) {
                        remaining.push(branch);
                        continue;
                    }
                }

                branch.create_in_work_dir(&temp_work_dir, &mut pushed_branches)?;
                created_order.push(branch);
                progress = true;
            }

            if !progress && !remaining.is_empty() {
                return Err(CreateError::GitError(format!(
                    "Cannot create branches: bases not found - {:?}",
                    remaining
                        .iter()
                        .filter_map(|b| b.base.clone())
                        .collect::<Vec<_>>()
                )));
            }

            branches_to_create = remaining;
        }

        std::fs::remove_dir_all(&temp_work_dir)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RemoteBranchDescriptor {
    name: String,
    base: Option<String>,
    commits: Vec<RemoteCommitDescriptor>,
}

impl RemoteBranchDescriptor {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            base: None,
            commits: Vec::new(),
        }
    }

    pub fn from(name: &str, base: &str) -> Self {
        Self {
            name: name.to_string(),
            base: Some(base.to_string()),
            commits: Vec::new(),
        }
    }

    pub fn with_commit(mut self, commit: RemoteCommitDescriptor) -> Self {
        self.commits.push(commit);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn base(&self) -> Option<&str> {
        self.base.as_deref()
    }

    fn create_in_work_dir(&self, work_dir: &PathBuf, pushed_branches: &mut std::collections::HashSet<String>) -> Result<(), CreateError> {
        validate_branch_name(&self.name)?;
        if let Some(base) = &self.base {
            validate_branch_name(base)?;
        }

        if let Some(base) = &self.base {
            let output = Command::new("git")
                .args(["checkout", base])
                .current_dir(work_dir)
                .output()?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(CreateError::GitError(format!("Failed to checkout base branch '{}': {}", base, stderr)));
            }
        }

        let output = Command::new("git")
            .args(["checkout", "-b", &self.name])
            .current_dir(work_dir)
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CreateError::GitError(format!("Failed to create branch '{}': {}", self.name, stderr)));
        }

        for commit in &self.commits {
            commit.apply(work_dir)?;
        }

        let output = Command::new("git")
            .args(["push", "origin", &self.name])
            .current_dir(work_dir)
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CreateError::GitError(format!("Failed to push branch '{}': {}", self.name, stderr)));
        }

        pushed_branches.insert(self.name.clone());

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RemoteCommitDescriptor {
    message: String,
    files: Vec<(String, String)>,
    deletes: Vec<String>,
}

impl RemoteCommitDescriptor {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            files: Vec::new(),
            deletes: Vec::new(),
        }
    }

    pub fn with_file(mut self, path: &str, content: &str) -> Self {
        self.files.push((path.to_string(), content.to_string()));
        self
    }

    pub fn with_delete(mut self, path: &str) -> Self {
        self.deletes.push(path.to_string());
        self
    }

    fn apply(&self, work_dir: &PathBuf) -> Result<(), CreateError> {
        for (path, content) in &self.files {
            let file_path = work_dir.join(path);
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&file_path, content)?;
        }

        for path in &self.deletes {
            let file_path = work_dir.join(path);
            if file_path.exists() {
                std::fs::remove_file(&file_path)?;
            }
        }

        Command::new("git")
            .args(["add", "-A"])
            .current_dir(work_dir)
            .output()?;

        let output = Command::new("git")
            .args(["commit", "-m", &self.message])
            .current_dir(work_dir)
            .output()?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CreateError::GitError(format!("Failed to commit: {}", stderr)));
        }

        Ok(())
    }
}
