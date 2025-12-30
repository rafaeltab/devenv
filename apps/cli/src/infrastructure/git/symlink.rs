//! Symlink management for git worktrees
//!
//! This module provides functionality to create symlinks from the main worktree
//! to new worktrees based on glob patterns.

use std::path::{Path, PathBuf};

use super::GitError;

/// Result of creating symlinks
#[derive(Debug, Clone)]
pub struct SymlinkResult {
    /// Files that were successfully symlinked
    pub created: Vec<PathBuf>,
    /// Files that were skipped (already exist in target or source doesn't exist)
    pub skipped: Vec<SkippedFile>,
}

/// Information about a file that was skipped during symlink creation
#[derive(Debug, Clone)]
pub struct SkippedFile {
    /// The file path
    pub path: PathBuf,
    /// Reason for skipping
    pub reason: SkipReason,
}

/// Reasons why a file might be skipped during symlink creation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// The source file doesn't exist
    SourceNotFound,
    /// The target file already exists
    TargetExists,
}

/// Create symlinks from source (main worktree) to target (new worktree) based on glob patterns.
///
/// Uses absolute paths for symlinks.
///
/// # Arguments
/// * `source_path` - Path to the main worktree (source of real files)
/// * `target_path` - Path to the new worktree (where symlinks will be created)
/// * `patterns` - Glob patterns for files to symlink
///
/// # Returns
/// A `SymlinkResult` containing information about created and skipped symlinks
pub fn create_symlinks(
    source_path: &Path,
    target_path: &Path,
    patterns: &[String],
) -> Result<SymlinkResult, GitError> {
    let mut result = SymlinkResult {
        created: Vec::new(),
        skipped: Vec::new(),
    };

    for pattern in patterns {
        let full_pattern = source_path.join(pattern);
        let pattern_str = full_pattern.to_string_lossy();

        // Use glob to find matching files
        let entries = match glob::glob(&pattern_str) {
            Ok(entries) => entries,
            Err(e) => {
                return Err(GitError::IoError(format!(
                    "Invalid glob pattern '{}': {}",
                    pattern, e
                )));
            }
        };

        for entry in entries {
            match entry {
                Ok(source_file) => {
                    // Calculate relative path from source_path
                    let relative = match source_file.strip_prefix(source_path) {
                        Ok(rel) => rel,
                        Err(_) => continue,
                    };

                    let target_file = target_path.join(relative);

                    // Skip if target already exists
                    if target_file.exists() || target_file.is_symlink() {
                        result.skipped.push(SkippedFile {
                            path: relative.to_path_buf(),
                            reason: SkipReason::TargetExists,
                        });
                        continue;
                    }

                    // Create parent directories if needed
                    if let Some(parent) = target_file.parent() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            return Err(GitError::IoError(format!(
                                "Failed to create directory {:?}: {}",
                                parent, e
                            )));
                        }
                    }

                    // Create symlink using absolute path
                    #[cfg(unix)]
                    {
                        if let Err(e) = std::os::unix::fs::symlink(&source_file, &target_file) {
                            return Err(GitError::IoError(format!(
                                "Failed to create symlink {:?} -> {:?}: {}",
                                target_file, source_file, e
                            )));
                        }
                    }

                    #[cfg(windows)]
                    {
                        // On Windows, we need to determine if it's a file or directory
                        let result = if source_file.is_dir() {
                            std::os::windows::fs::symlink_dir(&source_file, &target_file)
                        } else {
                            std::os::windows::fs::symlink_file(&source_file, &target_file)
                        };
                        
                        if let Err(e) = result {
                            return Err(GitError::IoError(format!(
                                "Failed to create symlink {:?} -> {:?}: {}",
                                target_file, source_file, e
                            )));
                        }
                    }

                    result.created.push(relative.to_path_buf());
                }
                Err(e) => {
                    // Glob error for this entry, skip it
                    eprintln!("Warning: glob error: {}", e);
                }
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_create_symlinks_with_simple_filename() {
        let source_dir = tempfile::tempdir().unwrap();
        let target_dir = tempfile::tempdir().unwrap();

        // Create source file
        fs::write(source_dir.path().join(".env"), "SECRET=value").unwrap();

        let result = create_symlinks(
            source_dir.path(),
            target_dir.path(),
            &[".env".to_string()],
        );

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.created.len(), 1);
        assert_eq!(result.created[0], PathBuf::from(".env"));

        // Verify symlink was created
        let symlink_path = target_dir.path().join(".env");
        assert!(symlink_path.is_symlink());
    }

    #[test]
    fn test_create_symlinks_with_glob_pattern() {
        let source_dir = tempfile::tempdir().unwrap();
        let target_dir = tempfile::tempdir().unwrap();

        // Create source files
        fs::write(source_dir.path().join("file1.secret.json"), "{}").unwrap();
        fs::write(source_dir.path().join("file2.secret.json"), "{}").unwrap();
        fs::write(source_dir.path().join("file.txt"), "not a secret").unwrap();

        let result = create_symlinks(
            source_dir.path(),
            target_dir.path(),
            &["*.secret.json".to_string()],
        );

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.created.len(), 2);
    }

    #[test]
    fn test_create_symlinks_with_nested_glob_pattern() {
        let source_dir = tempfile::tempdir().unwrap();
        let target_dir = tempfile::tempdir().unwrap();

        // Create nested source files
        let creds_dir = source_dir.path().join("credentials");
        fs::create_dir_all(&creds_dir).unwrap();
        fs::write(creds_dir.join("aws.json"), "{}").unwrap();
        fs::write(creds_dir.join("gcp.json"), "{}").unwrap();

        let result = create_symlinks(
            source_dir.path(),
            target_dir.path(),
            &["credentials/*".to_string()],
        );

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.created.len(), 2);

        // Verify symlinks were created in correct location
        assert!(target_dir.path().join("credentials/aws.json").is_symlink());
        assert!(target_dir.path().join("credentials/gcp.json").is_symlink());
    }

    #[test]
    fn test_create_symlinks_skips_nonexistent_files() {
        let source_dir = tempfile::tempdir().unwrap();
        let target_dir = tempfile::tempdir().unwrap();

        // Don't create any files

        let result = create_symlinks(
            source_dir.path(),
            target_dir.path(),
            &[".env".to_string()],
        );

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.created.is_empty());
        // No skipped files either since the pattern just didn't match anything
    }

    #[test]
    fn test_create_symlinks_does_not_overwrite_existing_files() {
        let source_dir = tempfile::tempdir().unwrap();
        let target_dir = tempfile::tempdir().unwrap();

        // Create source file
        fs::write(source_dir.path().join(".env"), "SECRET=value").unwrap();
        // Create existing target file
        fs::write(target_dir.path().join(".env"), "EXISTING=true").unwrap();

        let result = create_symlinks(
            source_dir.path(),
            target_dir.path(),
            &[".env".to_string()],
        );

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.created.is_empty());
        assert_eq!(result.skipped.len(), 1);
        assert_eq!(result.skipped[0].reason, SkipReason::TargetExists);

        // Verify original file is unchanged
        let content = fs::read_to_string(target_dir.path().join(".env")).unwrap();
        assert_eq!(content, "EXISTING=true");
    }

    #[test]
    fn test_create_symlinks_with_multiple_patterns() {
        let source_dir = tempfile::tempdir().unwrap();
        let target_dir = tempfile::tempdir().unwrap();

        // Create source files
        fs::write(source_dir.path().join(".env"), "SECRET=value").unwrap();
        fs::write(source_dir.path().join("config.json"), "{}").unwrap();

        let result = create_symlinks(
            source_dir.path(),
            target_dir.path(),
            &[".env".to_string(), "config.json".to_string()],
        );

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.created.len(), 2);
    }

    #[test]
    fn test_create_symlinks_with_empty_patterns_does_nothing() {
        let source_dir = tempfile::tempdir().unwrap();
        let target_dir = tempfile::tempdir().unwrap();

        let result = create_symlinks(source_dir.path(), target_dir.path(), &[]);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.created.is_empty());
        assert!(result.skipped.is_empty());
    }

    #[test]
    fn test_create_symlinks_uses_absolute_paths() {
        let source_dir = tempfile::tempdir().unwrap();
        let target_dir = tempfile::tempdir().unwrap();

        fs::write(source_dir.path().join(".env"), "SECRET=value").unwrap();

        let result = create_symlinks(
            source_dir.path(),
            target_dir.path(),
            &[".env".to_string()],
        );

        assert!(result.is_ok());

        // Read the symlink target and verify it's absolute
        let symlink_path = target_dir.path().join(".env");
        let link_target = fs::read_link(&symlink_path).unwrap();
        assert!(link_target.is_absolute());
    }
}
