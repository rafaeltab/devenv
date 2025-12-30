use serde::{Deserialize, Serialize};

use super::storage_interface::Storage;

/// Trait for storage that can read/write global worktree configuration
pub trait WorktreeStorage: Storage<Option<WorktreeConfig>> {}

/// Global worktree configuration that applies to all workspaces
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeConfig {
    /// Glob patterns for files to symlink from main worktree to new worktrees
    #[serde(default)]
    pub symlink_files: Vec<String>,
    /// Commands to run when creating a new worktree
    #[serde(default)]
    pub on_create: Vec<String>,
}

/// Per-workspace worktree configuration
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceWorktreeConfig {
    /// Glob patterns for files to symlink from main worktree to new worktrees
    /// These are merged with global symlink_files
    #[serde(default)]
    pub symlink_files: Vec<String>,
    /// Commands to run when creating a new worktree
    /// These are merged with global on_create commands
    #[serde(default)]
    pub on_create: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_global_worktree_config() {
        let json = r#"{
            "symlinkFiles": [".env", "*.secret.json"],
            "onCreate": ["npm install"]
        }"#;

        let config: WorktreeConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.symlink_files, vec![".env", "*.secret.json"]);
        assert_eq!(config.on_create, vec!["npm install"]);
    }

    #[test]
    fn test_deserialize_workspace_worktree_config() {
        let json = r#"{
            "symlinkFiles": [".env.local", "credentials/*"],
            "onCreate": ["pnpm install", "cp .env.example .env"]
        }"#;

        let config: WorkspaceWorktreeConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.symlink_files, vec![".env.local", "credentials/*"]);
        assert_eq!(config.on_create, vec!["pnpm install", "cp .env.example .env"]);
    }

    #[test]
    fn test_deserialize_worktree_config_with_empty_arrays() {
        let json = r#"{
            "symlinkFiles": [],
            "onCreate": []
        }"#;

        let config: WorktreeConfig = serde_json::from_str(json).unwrap();

        assert!(config.symlink_files.is_empty());
        assert!(config.on_create.is_empty());
    }

    #[test]
    fn test_deserialize_worktree_config_with_missing_fields() {
        let json = r#"{}"#;

        let config: WorktreeConfig = serde_json::from_str(json).unwrap();

        assert!(config.symlink_files.is_empty());
        assert!(config.on_create.is_empty());
    }

    #[test]
    fn test_deserialize_worktree_config_with_partial_fields() {
        let json = r#"{
            "symlinkFiles": [".env"]
        }"#;

        let config: WorktreeConfig = serde_json::from_str(json).unwrap();

        assert_eq!(config.symlink_files, vec![".env"]);
        assert!(config.on_create.is_empty());
    }

    #[test]
    fn test_serialize_worktree_config() {
        let config = WorktreeConfig {
            symlink_files: vec![".env".to_string()],
            on_create: vec!["npm install".to_string()],
        };

        let json = serde_json::to_string(&config).unwrap();

        assert!(json.contains("symlinkFiles"));
        assert!(json.contains(".env"));
        assert!(json.contains("onCreate"));
        assert!(json.contains("npm install"));
    }

    #[test]
    fn test_default_worktree_config() {
        let config = WorktreeConfig::default();

        assert!(config.symlink_files.is_empty());
        assert!(config.on_create.is_empty());
    }

    #[test]
    fn test_default_workspace_worktree_config() {
        let config = WorkspaceWorktreeConfig::default();

        assert!(config.symlink_files.is_empty());
        assert!(config.on_create.is_empty());
    }
}
