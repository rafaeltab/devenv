//! Configuration merging and handling for worktrees

use std::path::{Path, PathBuf};

use crate::storage::worktree::{WorkspaceWorktreeConfig, WorktreeConfig};

/// Merged worktree configuration from global and workspace-specific settings
#[derive(Debug, Clone, Default)]
pub struct MergedWorktreeConfig {
    /// Combined symlink file patterns (global + workspace)
    pub symlink_files: Vec<String>,
    /// Combined onCreate commands (global + workspace)
    pub on_create: Vec<String>,
}

impl MergedWorktreeConfig {
    /// Create a new merged config from global and workspace configs.
    ///
    /// Uses union strategy - combines both lists, with workspace-specific items added after global.
    pub fn merge(
        global: Option<&WorktreeConfig>,
        workspace: Option<&WorkspaceWorktreeConfig>,
    ) -> Self {
        let mut symlink_files = Vec::new();
        let mut on_create = Vec::new();

        // Add global config first
        if let Some(global_config) = global {
            symlink_files.extend(global_config.symlink_files.clone());
            on_create.extend(global_config.on_create.clone());
        }

        // Add workspace-specific config (these come after global)
        if let Some(workspace_config) = workspace {
            // Deduplicate while preserving order
            for file in &workspace_config.symlink_files {
                if !symlink_files.contains(file) {
                    symlink_files.push(file.clone());
                }
            }
            for cmd in &workspace_config.on_create {
                if !on_create.contains(cmd) {
                    on_create.push(cmd.clone());
                }
            }
        }

        MergedWorktreeConfig {
            symlink_files,
            on_create,
        }
    }

    /// Check if this config is empty (no symlink files and no onCreate commands)
    pub fn is_empty(&self) -> bool {
        self.symlink_files.is_empty() && self.on_create.is_empty()
    }
}

/// Calculate the worktree path for a given branch name.
///
/// The worktree is created as a sibling of the main workspace root.
/// For branch names with slashes (e.g., "feat/user/login"), nested directories are created.
///
/// # Arguments
/// * `workspace_root` - The root path of the main workspace
/// * `branch_name` - The name of the branch (can contain slashes)
///
/// # Returns
/// The absolute path where the worktree should be created
pub fn calculate_worktree_path(workspace_root: &Path, branch_name: &str) -> PathBuf {
    // Get the parent directory of the workspace root
    let parent = workspace_root
        .parent()
        .unwrap_or(workspace_root);

    // Join with the branch name (slashes in branch name create nested directories)
    parent.join(branch_name)
}

/// Information about a worktree to be created
#[derive(Debug, Clone)]
pub struct WorktreeCreationInfo {
    /// The branch name for the worktree
    pub branch_name: String,
    /// The base branch to create from (current branch)
    pub base_branch: String,
    /// Whether the branch is new, local, or remote
    pub branch_status: BranchStatus,
    /// The path where the worktree will be created
    pub worktree_path: PathBuf,
    /// Merged configuration for symlinks and onCreate
    pub config: MergedWorktreeConfig,
    /// The workspace name
    pub workspace_name: String,
}

/// Status of a branch for display purposes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchStatus {
    /// Branch will be created new
    New,
    /// Branch exists locally
    ExistsLocally,
    /// Branch exists on remote
    ExistsRemotely(String),
}

impl std::fmt::Display for BranchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BranchStatus::New => write!(f, "New branch (will be created)"),
            BranchStatus::ExistsLocally => write!(f, "Existing branch (local)"),
            BranchStatus::ExistsRemotely(remote) => {
                write!(f, "Existing branch (remote {}/...)", remote)
            }
        }
    }
}

/// Find the workspace that contains the given path.
/// When workspaces are nested, returns the most specific (longest path) match.
///
/// # Arguments
/// * `current_path` - The current directory path (must be an absolute expanded path)
/// * `workspace_paths` - Iterator of (workspace, expanded_path) tuples
///
/// # Returns
/// The workspace with the longest matching path, or None if no match
pub fn find_most_specific_workspace<'a, I>(current_path: &str, workspace_paths: I) -> Option<&'a str>
where
    I: Iterator<Item = (&'a str, &'a str)>,
{
    workspace_paths
        .filter(|(_, ws_path)| current_path.starts_with(*ws_path))
        .max_by_key(|(_, ws_path)| ws_path.len())
        .map(|(id, _)| id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_configs_combines_global_and_workspace_symlinks() {
        let global = WorktreeConfig {
            symlink_files: vec![".env".to_string()],
            on_create: vec![],
        };
        let workspace = WorkspaceWorktreeConfig {
            symlink_files: vec!["secrets.json".to_string()],
            on_create: vec![],
        };

        let result = MergedWorktreeConfig::merge(Some(&global), Some(&workspace));

        assert_eq!(result.symlink_files, vec![".env", "secrets.json"]);
    }

    #[test]
    fn test_merge_configs_combines_global_and_workspace_on_create() {
        let global = WorktreeConfig {
            symlink_files: vec![],
            on_create: vec!["npm install".to_string()],
        };
        let workspace = WorkspaceWorktreeConfig {
            symlink_files: vec![],
            on_create: vec!["npm run build".to_string()],
        };

        let result = MergedWorktreeConfig::merge(Some(&global), Some(&workspace));

        assert_eq!(result.on_create, vec!["npm install", "npm run build"]);
    }

    #[test]
    fn test_merge_configs_deduplicates_symlinks() {
        let global = WorktreeConfig {
            symlink_files: vec![".env".to_string(), "config.json".to_string()],
            on_create: vec![],
        };
        let workspace = WorkspaceWorktreeConfig {
            symlink_files: vec![".env".to_string(), "secrets.json".to_string()],
            on_create: vec![],
        };

        let result = MergedWorktreeConfig::merge(Some(&global), Some(&workspace));

        assert_eq!(
            result.symlink_files,
            vec![".env", "config.json", "secrets.json"]
        );
    }

    #[test]
    fn test_merge_configs_with_no_global_config() {
        let workspace = WorkspaceWorktreeConfig {
            symlink_files: vec![".env".to_string()],
            on_create: vec!["npm install".to_string()],
        };

        let result = MergedWorktreeConfig::merge(None, Some(&workspace));

        assert_eq!(result.symlink_files, vec![".env"]);
        assert_eq!(result.on_create, vec!["npm install"]);
    }

    #[test]
    fn test_merge_configs_with_no_workspace_config() {
        let global = WorktreeConfig {
            symlink_files: vec![".env".to_string()],
            on_create: vec!["npm install".to_string()],
        };

        let result = MergedWorktreeConfig::merge(Some(&global), None);

        assert_eq!(result.symlink_files, vec![".env"]);
        assert_eq!(result.on_create, vec!["npm install"]);
    }

    #[test]
    fn test_merge_configs_with_neither_config() {
        let result = MergedWorktreeConfig::merge(None, None);

        assert!(result.symlink_files.is_empty());
        assert!(result.on_create.is_empty());
        assert!(result.is_empty());
    }

    #[test]
    fn test_get_worktree_path_simple_branch_name() {
        let workspace_root = PathBuf::from("/home/user/source/myproject");

        let result = calculate_worktree_path(&workspace_root, "feature-branch");

        assert_eq!(result, PathBuf::from("/home/user/source/feature-branch"));
    }

    #[test]
    fn test_get_worktree_path_with_slashes_in_branch() {
        let workspace_root = PathBuf::from("/home/user/source/myproject");

        let result = calculate_worktree_path(&workspace_root, "feat/user/login");

        assert_eq!(
            result,
            PathBuf::from("/home/user/source/feat/user/login")
        );
    }

    #[test]
    fn test_get_worktree_path_deeply_nested_branch() {
        let workspace_root = PathBuf::from("/home/user/source/myproject");

        let result = calculate_worktree_path(&workspace_root, "feat/scope/area/specific-fix");

        assert_eq!(
            result,
            PathBuf::from("/home/user/source/feat/scope/area/specific-fix")
        );
    }

    #[test]
    fn test_branch_status_display_new() {
        let status = BranchStatus::New;
        assert_eq!(format!("{}", status), "New branch (will be created)");
    }

    #[test]
    fn test_branch_status_display_local() {
        let status = BranchStatus::ExistsLocally;
        assert_eq!(format!("{}", status), "Existing branch (local)");
    }

    #[test]
    fn test_branch_status_display_remote() {
        let status = BranchStatus::ExistsRemotely("origin".to_string());
        assert_eq!(format!("{}", status), "Existing branch (remote origin/...)");
    }

    #[test]
    fn test_merged_config_is_empty_when_empty() {
        let config = MergedWorktreeConfig::default();
        assert!(config.is_empty());
    }

    #[test]
    fn test_merged_config_is_not_empty_with_symlinks() {
        let config = MergedWorktreeConfig {
            symlink_files: vec![".env".to_string()],
            on_create: vec![],
        };
        assert!(!config.is_empty());
    }

    #[test]
    fn test_merged_config_is_not_empty_with_on_create() {
        let config = MergedWorktreeConfig {
            symlink_files: vec![],
            on_create: vec!["npm install".to_string()],
        };
        assert!(!config.is_empty());
    }

    #[test]
    fn test_find_most_specific_workspace_simple_match() {
        let workspaces = vec![
            ("devenv", "/home/user/source/devenv"),
        ];
        
        let result = find_most_specific_workspace(
            "/home/user/source/devenv/apps/cli",
            workspaces.iter().map(|(id, path)| (*id, *path)),
        );
        
        assert_eq!(result, Some("devenv"));
    }

    #[test]
    fn test_find_most_specific_workspace_nested_returns_most_specific() {
        let workspaces = vec![
            ("home", "/home/user"),
            ("devenv", "/home/user/source/devenv"),
        ];
        
        let result = find_most_specific_workspace(
            "/home/user/source/devenv/apps/cli",
            workspaces.iter().map(|(id, path)| (*id, *path)),
        );
        
        assert_eq!(result, Some("devenv"));
    }

    #[test]
    fn test_find_most_specific_workspace_nested_parent_first_in_list() {
        // Test that order in the list doesn't matter
        let workspaces = vec![
            ("devenv", "/home/user/source/devenv"),
            ("home", "/home/user"),
        ];
        
        let result = find_most_specific_workspace(
            "/home/user/source/devenv/apps/cli",
            workspaces.iter().map(|(id, path)| (*id, *path)),
        );
        
        assert_eq!(result, Some("devenv"));
    }

    #[test]
    fn test_find_most_specific_workspace_returns_parent_when_not_in_child() {
        let workspaces = vec![
            ("home", "/home/user"),
            ("devenv", "/home/user/source/devenv"),
        ];
        
        let result = find_most_specific_workspace(
            "/home/user/documents",
            workspaces.iter().map(|(id, path)| (*id, *path)),
        );
        
        assert_eq!(result, Some("home"));
    }

    #[test]
    fn test_find_most_specific_workspace_no_match() {
        let workspaces = vec![
            ("devenv", "/home/user/source/devenv"),
        ];
        
        let result = find_most_specific_workspace(
            "/tmp/other",
            workspaces.iter().map(|(id, path)| (*id, *path)),
        );
        
        assert_eq!(result, None);
    }

    #[test]
    fn test_find_most_specific_workspace_deeply_nested() {
        let workspaces = vec![
            ("home", "/home/user"),
            ("source", "/home/user/source"),
            ("devenv", "/home/user/source/devenv"),
        ];
        
        let result = find_most_specific_workspace(
            "/home/user/source/devenv/apps/cli/src",
            workspaces.iter().map(|(id, path)| (*id, *path)),
        );
        
        assert_eq!(result, Some("devenv"));
    }
}
