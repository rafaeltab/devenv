use crate::environment::TestEnvironment;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GitRepoRef<'a> {
    pub(crate) name: String,
    pub(crate) path: PathBuf,
    pub(crate) env: &'a TestEnvironment,
}

impl<'a> GitRepoRef<'a> {
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn exists(&self) -> bool {
        self.path.exists() && self.path.join(".git").exists()
    }

    pub fn current_branch(&self) -> String {
        let output = Command::new("git")
            .args(["branch", "--show-current"])
            .current_dir(&self.path)
            .output()
            .expect("Failed to run git branch");

        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    pub fn branches(&self) -> Vec<String> {
        let output = Command::new("git")
            .args(["branch", "--list", "--format=%(refname:short)"])
            .current_dir(&self.path)
            .output()
            .expect("Failed to run git branch");

        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn is_clean(&self) -> bool {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.path)
            .output()
            .expect("Failed to run git status");

        output.stdout.is_empty()
    }

    pub fn has_staged_changes(&self) -> bool {
        let output = Command::new("git")
            .args(["diff", "--cached", "--name-only"])
            .current_dir(&self.path)
            .output()
            .expect("Failed to run git diff --cached");

        !output.stdout.is_empty()
    }

    pub fn has_unstaged_changes(&self) -> bool {
        let output = Command::new("git")
            .args(["diff", "--name-only"])
            .current_dir(&self.path)
            .output()
            .expect("Failed to run git diff");

        !output.stdout.is_empty()
    }

    pub fn has_untracked_files(&self) -> bool {
        let output = Command::new("git")
            .args(["ls-files", "--others", "--exclude-standard"])
            .current_dir(&self.path)
            .output()
            .expect("Failed to run git ls-files");

        !output.stdout.is_empty()
    }

    pub fn has_unpushed_commits(&self) -> bool {
        // Check if there's an upstream; if not, all commits are "unpushed"
        let branch = self.current_branch();
        let upstream_check = Command::new("git")
            .args([
                "rev-parse",
                "--abbrev-ref",
                &format!("{}@{{upstream}}", branch),
            ])
            .current_dir(&self.path)
            .output();

        match upstream_check {
            Ok(output) if output.status.success() => {
                // Has upstream, check for unpushed
                let log_output = Command::new("git")
                    .args(["log", "@{upstream}..HEAD", "--oneline"])
                    .current_dir(&self.path)
                    .output()
                    .expect("Failed to run git log");

                !log_output.stdout.is_empty()
            }
            _ => {
                // No upstream, consider all commits as unpushed
                true
            }
        }
    }

    /// Run an arbitrary git command in this repo
    pub fn git(&self, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.path)
            .output()
            .expect("Failed to run git command");

        String::from_utf8_lossy(&output.stdout).to_string()
    }
}
