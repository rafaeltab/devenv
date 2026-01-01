use test_descriptors::TestEnvironment;

// ============================================================================
// Staged Changes Tests (Git Repo)
// ============================================================================

#[test]
fn test_git_repo_with_staged_file() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("my-repo", |g| {
                g.branch("main", |b| {
                    b.commit("Initial", |c| {
                        c.file("README.md", "# Test");
                    });
                });
                g.staged(|s| {
                    s.file("staged.txt", "staged content");
                });
            });
        });
    })
    .create();

    let repo = env.find_git_repo("my-repo").expect("repo should exist");

    // Should have staged changes
    assert!(repo.has_staged_changes());
    // Should not have unstaged changes
    assert!(!repo.has_unstaged_changes());
    // Should not be clean (has staged changes)
    assert!(!repo.is_clean());
    // File should exist on disk
    assert!(repo.path().join("staged.txt").exists());
}

#[test]
fn test_git_repo_with_staged_deletion() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("my-repo", |g| {
                g.branch("main", |b| {
                    b.commit("Initial", |c| {
                        c.file("README.md", "# Test");
                        c.file("to-delete.txt", "will be deleted");
                    });
                });
                g.staged(|s| {
                    s.delete("to-delete.txt");
                });
            });
        });
    })
    .create();

    let repo = env.find_git_repo("my-repo").expect("repo should exist");

    // Should have staged changes (the deletion)
    assert!(repo.has_staged_changes());
    // File should not exist on disk
    assert!(!repo.path().join("to-delete.txt").exists());
    // README should still exist
    assert!(repo.path().join("README.md").exists());
}

#[test]
fn test_git_repo_with_multiple_staged_files() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("my-repo", |g| {
                g.branch("main", |b| {
                    b.commit("Initial", |c| {
                        c.file("README.md", "# Test");
                    });
                });
                g.staged(|s| {
                    s.file("file1.txt", "content 1");
                    s.file("file2.txt", "content 2");
                    s.file("src/file3.txt", "content 3");
                });
            });
        });
    })
    .create();

    let repo = env.find_git_repo("my-repo").expect("repo should exist");

    assert!(repo.has_staged_changes());
    assert!(repo.path().join("file1.txt").exists());
    assert!(repo.path().join("file2.txt").exists());
    assert!(repo.path().join("src/file3.txt").exists());
}

// ============================================================================
// Unstaged Changes Tests (Git Repo)
// ============================================================================

#[test]
fn test_git_repo_with_unstaged_modification() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("my-repo", |g| {
                g.branch("main", |b| {
                    b.commit("Initial", |c| {
                        c.file("README.md", "# Original");
                    });
                });
                g.unstaged(|u| {
                    u.modify("README.md", "# Modified");
                });
            });
        });
    })
    .create();

    let repo = env.find_git_repo("my-repo").expect("repo should exist");

    // Should have unstaged changes
    assert!(repo.has_unstaged_changes());
    // Should not have staged changes
    assert!(!repo.has_staged_changes());
    // Should not be clean
    assert!(!repo.is_clean());
    // File should have new content
    let content = std::fs::read_to_string(repo.path().join("README.md")).unwrap();
    assert_eq!(content, "# Modified");
}

#[test]
fn test_git_repo_with_untracked_file() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("my-repo", |g| {
                g.branch("main", |b| {
                    b.commit("Initial", |c| {
                        c.file("README.md", "# Test");
                    });
                });
                g.unstaged(|u| {
                    u.untracked("new-file.txt", "untracked content");
                });
            });
        });
    })
    .create();

    let repo = env.find_git_repo("my-repo").expect("repo should exist");

    // Should have untracked files
    assert!(repo.has_untracked_files());
    // Untracked files don't show as unstaged changes (modifications)
    assert!(!repo.has_unstaged_changes());
    // Should not have staged changes
    assert!(!repo.has_staged_changes());
    // Should not be clean (untracked files make it dirty)
    assert!(!repo.is_clean());
    // File should exist on disk
    assert!(repo.path().join("new-file.txt").exists());
}

#[test]
fn test_git_repo_with_unstaged_deletion() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("my-repo", |g| {
                g.branch("main", |b| {
                    b.commit("Initial", |c| {
                        c.file("README.md", "# Test");
                        c.file("to-delete.txt", "will be deleted");
                    });
                });
                g.unstaged(|u| {
                    u.delete("to-delete.txt");
                });
            });
        });
    })
    .create();

    let repo = env.find_git_repo("my-repo").expect("repo should exist");

    // Should have unstaged changes (the deletion)
    assert!(repo.has_unstaged_changes());
    // Should not have staged changes
    assert!(!repo.has_staged_changes());
    // File should not exist on disk
    assert!(!repo.path().join("to-delete.txt").exists());
}

// ============================================================================
// Combined Staged + Unstaged Tests
// ============================================================================

#[test]
fn test_git_repo_with_both_staged_and_unstaged() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("my-repo", |g| {
                g.branch("main", |b| {
                    b.commit("Initial", |c| {
                        c.file("README.md", "# Original");
                    });
                });
                g.staged(|s| {
                    s.file("staged.txt", "staged content");
                });
                g.unstaged(|u| {
                    u.modify("README.md", "# Modified");
                    u.untracked("untracked.txt", "untracked content");
                });
            });
        });
    })
    .create();

    let repo = env.find_git_repo("my-repo").expect("repo should exist");

    // Should have both staged and unstaged changes
    assert!(repo.has_staged_changes());
    assert!(repo.has_unstaged_changes());
    assert!(repo.has_untracked_files());
    assert!(!repo.is_clean());

    // All files should exist
    assert!(repo.path().join("staged.txt").exists());
    assert!(repo.path().join("untracked.txt").exists());

    // README should have modified content
    let content = std::fs::read_to_string(repo.path().join("README.md")).unwrap();
    assert_eq!(content, "# Modified");
}

// ============================================================================
// Worktree Staged/Unstaged Tests
// ============================================================================

#[test]
fn test_worktree_with_staged_changes() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("my-repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Test");
                        });
                    });
                });
                d.git_worktree("my-repo", "main", "feature/test", |w| {
                    w.commit("Add feature", |c| {
                        c.file("feature.txt", "feature");
                    });
                    w.staged(|s| {
                        s.file("staged-in-wt.txt", "staged in worktree");
                    });
                });
            });
        });
    })
    .create();

    let worktree = env
        .find_worktree("my-repo", "feature/test")
        .expect("worktree should exist");

    assert!(worktree.has_staged_changes());
    assert!(!worktree.has_unstaged_changes());
    assert!(worktree.path().join("staged-in-wt.txt").exists());
}

#[test]
fn test_worktree_with_unstaged_changes() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("my-repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Original");
                        });
                    });
                });
                d.git_worktree("my-repo", "main", "feature/test", |w| {
                    w.unstaged(|u| {
                        u.modify("README.md", "# Modified in worktree");
                        u.untracked("untracked-wt.txt", "untracked in worktree");
                    });
                });
            });
        });
    })
    .create();

    let worktree = env
        .find_worktree("my-repo", "feature/test")
        .expect("worktree should exist");

    assert!(worktree.has_unstaged_changes());
    assert!(worktree.has_untracked_files());
    assert!(!worktree.has_staged_changes());

    let content = std::fs::read_to_string(worktree.path().join("README.md")).unwrap();
    assert_eq!(content, "# Modified in worktree");
}

#[test]
fn test_worktree_with_both_staged_and_unstaged() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("my-repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Test");
                        });
                    });
                });
                d.git_worktree("my-repo", "main", "feature/test", |w| {
                    w.staged(|s| {
                        s.file("staged.txt", "staged");
                    });
                    w.unstaged(|u| {
                        u.untracked("untracked.txt", "untracked");
                    });
                });
            });
        });
    })
    .create();

    let worktree = env
        .find_worktree("my-repo", "feature/test")
        .expect("worktree should exist");

    assert!(worktree.has_staged_changes());
    assert!(worktree.has_untracked_files());
    assert!(!worktree.is_clean());
}

// ============================================================================
// Main repo stays clean when worktree has changes
// ============================================================================

#[test]
fn test_main_repo_clean_when_worktree_dirty() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("my-repo", |_g| {});
                d.git_worktree("my-repo", "main", "feature/dirty", |w| {
                    w.staged(|s| {
                        s.file("staged.txt", "staged in worktree");
                    });
                    w.unstaged(|u| {
                        u.untracked("untracked.txt", "untracked in worktree");
                    });
                });
            });
        });
    })
    .create();

    let repo = env.find_git_repo("my-repo").expect("repo should exist");
    let worktree = env
        .find_worktree("my-repo", "feature/dirty")
        .expect("worktree should exist");

    // Main repo should be clean
    assert!(repo.is_clean());
    assert!(!repo.has_staged_changes());
    assert!(!repo.has_unstaged_changes());

    // Worktree should be dirty
    assert!(!worktree.is_clean());
    assert!(worktree.has_staged_changes());
    assert!(worktree.has_untracked_files());
}
