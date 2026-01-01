use test_descriptors::TestEnvironment;

// ============================================================================
// Directory Structure Tests
// ============================================================================

#[test]
fn test_worktree_is_sibling_to_repo_in_same_dir() {
    // When worktree is created in the same dir as the repo,
    // they should be siblings:
    //   workspace/
    //     my-repo/
    //     feature-test/
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
                d.git_worktree("my-repo", "main", "feature/test", |_w| {});
            });
        });
    })
    .create();

    let repo = env.find_git_repo("my-repo").expect("repo should exist");
    let worktree = env
        .find_worktree("my-repo", "feature/test")
        .expect("worktree should exist");
    let workspace = env.find_dir("workspace").expect("workspace should exist");

    // Both should be inside workspace
    assert!(repo.path().starts_with(workspace.path()));
    assert!(worktree.path().starts_with(workspace.path()));

    // They should be siblings (same parent)
    assert_eq!(repo.path().parent(), worktree.path().parent());

    // Verify exact structure
    assert_eq!(repo.path(), workspace.path().join("my-repo"));
    assert_eq!(worktree.path(), workspace.path().join("feature-test"));

    // Verify filesystem structure actually exists
    let root = env.root_path();
    assert!(
        root.join("workspace").is_dir(),
        "workspace/ should exist on disk"
    );
    assert!(
        root.join("workspace/my-repo").is_dir(),
        "workspace/my-repo/ should exist on disk"
    );
    assert!(
        root.join("workspace/my-repo/.git").exists(),
        "workspace/my-repo/.git should exist"
    );
    assert!(
        root.join("workspace/feature-test").is_dir(),
        "workspace/feature-test/ should exist on disk"
    );
    assert!(
        root.join("workspace/feature-test/.git").exists(),
        "workspace/feature-test/.git should exist"
    );
}

#[test]
fn test_worktree_in_separate_directory() {
    // Worktree can be created in a different directory than the repo:
    //   repos/
    //     my-repo/
    //   worktrees/
    //     feature-test/
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("repos", |d| {
                d.git("my-repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("README.md", "# Test");
                        });
                    });
                });
            });
            td.dir("worktrees", |d| {
                d.git_worktree("my-repo", "main", "feature/test", |_w| {});
            });
        });
    })
    .create();

    let repo = env.find_git_repo("my-repo").expect("repo should exist");
    let worktree = env
        .find_worktree("my-repo", "feature/test")
        .expect("worktree should exist");
    let repos_dir = env.find_dir("repos").expect("repos dir should exist");
    let worktrees_dir = env
        .find_dir("worktrees")
        .expect("worktrees dir should exist");

    // Repo should be in repos/
    assert_eq!(repo.path(), repos_dir.path().join("my-repo"));

    // Worktree should be in worktrees/
    assert_eq!(worktree.path(), worktrees_dir.path().join("feature-test"));

    // They should have different parents
    assert_ne!(repo.path().parent(), worktree.path().parent());

    // Verify filesystem structure actually exists
    let root = env.root_path();
    assert!(root.join("repos").is_dir(), "repos/ should exist on disk");
    assert!(
        root.join("repos/my-repo").is_dir(),
        "repos/my-repo/ should exist on disk"
    );
    assert!(
        root.join("repos/my-repo/.git").exists(),
        "repos/my-repo/.git should exist"
    );
    assert!(
        root.join("worktrees").is_dir(),
        "worktrees/ should exist on disk"
    );
    assert!(
        root.join("worktrees/feature-test").is_dir(),
        "worktrees/feature-test/ should exist on disk"
    );
    assert!(
        root.join("worktrees/feature-test/.git").exists(),
        "worktrees/feature-test/.git should exist"
    );
}

#[test]
fn test_worktree_in_nested_directory() {
    // Worktree can be deeply nested:
    //   project/
    //     src/
    //       my-repo/
    //     features/
    //       active/
    //         feature-test/
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("project", |project| {
                project.dir("src", |src| {
                    src.git("my-repo", |g| {
                        g.branch("main", |b| {
                            b.commit("Initial", |c| {
                                c.file("README.md", "# Test");
                            });
                        });
                    });
                });
                project.dir("features", |features| {
                    features.dir("active", |active| {
                        active.git_worktree("my-repo", "main", "feature/test", |_w| {});
                    });
                });
            });
        });
    })
    .create();

    let repo = env.find_git_repo("my-repo").expect("repo should exist");
    let worktree = env
        .find_worktree("my-repo", "feature/test")
        .expect("worktree should exist");

    // Verify the nested structure via query API
    assert!(repo
        .path()
        .to_string_lossy()
        .contains("project/src/my-repo"));
    assert!(worktree
        .path()
        .to_string_lossy()
        .contains("project/features/active/feature-test"));

    // Verify filesystem structure actually exists
    let root = env.root_path();
    assert!(
        root.join("project").is_dir(),
        "project/ should exist on disk"
    );
    assert!(
        root.join("project/src").is_dir(),
        "project/src/ should exist on disk"
    );
    assert!(
        root.join("project/src/my-repo").is_dir(),
        "project/src/my-repo/ should exist on disk"
    );
    assert!(
        root.join("project/src/my-repo/.git").exists(),
        "project/src/my-repo/.git should exist"
    );
    assert!(
        root.join("project/features").is_dir(),
        "project/features/ should exist on disk"
    );
    assert!(
        root.join("project/features/active").is_dir(),
        "project/features/active/ should exist on disk"
    );
    assert!(
        root.join("project/features/active/feature-test").is_dir(),
        "project/features/active/feature-test/ should exist on disk"
    );
    assert!(
        root.join("project/features/active/feature-test/.git")
            .exists(),
        "project/features/active/feature-test/.git should exist"
    );
}

// ============================================================================
// Basic Functionality Tests
// ============================================================================

#[test]
fn test_worktree_creates_directory() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("my-repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial commit", |c| {
                            c.file("README.md", "# My Repo");
                        });
                    });
                });
                d.git_worktree("my-repo", "main", "feature/test", |_w| {});
            });
        });
    })
    .create();

    let worktree = env
        .find_worktree("my-repo", "feature/test")
        .expect("worktree should exist");

    assert!(worktree.exists());
    assert!(worktree.path().ends_with("feature-test"));
}

#[test]
fn test_worktree_is_on_correct_branch() {
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
                d.git_worktree("my-repo", "main", "feature/awesome", |_w| {});
            });
        });
    })
    .create();

    let worktree = env
        .find_worktree("my-repo", "feature/awesome")
        .expect("worktree should exist");

    assert_eq!(worktree.current_branch(), "feature/awesome");
}

#[test]
fn test_worktree_inherits_base_branch_content() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("my-repo", |g| {
                    g.branch("develop", |b| {
                        b.commit("Add config", |c| {
                            c.file("config.json", "{}");
                        });
                        b.commit("Add src", |c| {
                            c.file("src/main.rs", "fn main() {}");
                        });
                    });
                });
                d.git_worktree("my-repo", "develop", "feature/new-feature", |_w| {});
            });
        });
    })
    .create();

    let worktree = env
        .find_worktree("my-repo", "feature/new-feature")
        .expect("worktree should exist");

    // Worktree should have files from develop branch
    assert!(worktree.path().join("config.json").exists());
    assert!(worktree.path().join("src/main.rs").exists());
}

#[test]
fn test_worktree_with_commits() {
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
                d.git_worktree("my-repo", "main", "feature/add-tests", |w| {
                    w.commit("Add test file", |c| {
                        c.file("tests/test.rs", "// tests");
                    });
                    w.commit("Add another test", |c| {
                        c.file("tests/test2.rs", "// more tests");
                    });
                });
            });
        });
    })
    .create();

    let worktree = env
        .find_worktree("my-repo", "feature/add-tests")
        .expect("worktree should exist");

    // Worktree should have the committed files
    assert!(worktree.path().join("tests/test.rs").exists());
    assert!(worktree.path().join("tests/test2.rs").exists());

    // Should have commits from main + 2 new commits
    // main has: Initial commit + README
    // worktree has: those + 2 more
    assert!(worktree.commit_count() >= 3);
}

#[test]
fn test_worktree_is_clean_after_creation() {
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
                d.git_worktree("my-repo", "main", "feature/clean", |w| {
                    w.commit("Add file", |c| {
                        c.file("file.txt", "content");
                    });
                });
            });
        });
    })
    .create();

    let worktree = env
        .find_worktree("my-repo", "feature/clean")
        .expect("worktree should exist");

    assert!(worktree.is_clean());
    assert!(!worktree.has_staged_changes());
    assert!(!worktree.has_unstaged_changes());
}

#[test]
fn test_multiple_worktrees_from_same_repo() {
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
                d.git_worktree("my-repo", "main", "feature/one", |w| {
                    w.commit("Feature one", |c| {
                        c.file("one.txt", "one");
                    });
                });
                d.git_worktree("my-repo", "main", "feature/two", |w| {
                    w.commit("Feature two", |c| {
                        c.file("two.txt", "two");
                    });
                });
            });
        });
    })
    .create();

    let wt1 = env
        .find_worktree("my-repo", "feature/one")
        .expect("worktree one should exist");
    let wt2 = env
        .find_worktree("my-repo", "feature/two")
        .expect("worktree two should exist");

    assert!(wt1.exists());
    assert!(wt2.exists());

    // Each worktree has its own files
    assert!(wt1.path().join("one.txt").exists());
    assert!(!wt1.path().join("two.txt").exists());

    assert!(wt2.path().join("two.txt").exists());
    assert!(!wt2.path().join("one.txt").exists());

    // Different branches
    assert_eq!(wt1.current_branch(), "feature/one");
    assert_eq!(wt2.current_branch(), "feature/two");
}

#[test]
fn test_worktree_from_different_base_branch() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("my-repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Main commit", |c| {
                            c.file("main.txt", "main content");
                        });
                    });
                    g.branch("develop", |b| {
                        b.commit("Develop commit", |c| {
                            c.file("develop.txt", "develop content");
                        });
                    });
                });
                // Worktree from develop, not main
                d.git_worktree("my-repo", "develop", "feature/from-develop", |_w| {});
            });
        });
    })
    .create();

    let worktree = env
        .find_worktree("my-repo", "feature/from-develop")
        .expect("worktree should exist");

    // Should have develop.txt but not main.txt (since develop doesn't inherit from main in this case)
    assert!(worktree.path().join("develop.txt").exists());
}

#[test]
fn test_worktree_query_returns_none_for_nonexistent() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("my-repo", |_g| {});
            });
        });
    })
    .create();

    assert!(env.find_worktree("my-repo", "nonexistent").is_none());
    assert!(env.find_worktree("nonexistent-repo", "main").is_none());
}

#[test]
fn test_worktree_with_file_deletion() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("my-repo", |g| {
                    g.branch("main", |b| {
                        b.commit("Initial", |c| {
                            c.file("keep.txt", "keep");
                            c.file("delete.txt", "delete");
                        });
                    });
                });
                d.git_worktree("my-repo", "main", "feature/cleanup", |w| {
                    w.commit("Remove file", |c| {
                        c.delete("delete.txt");
                    });
                });
            });
        });
    })
    .create();

    let worktree = env
        .find_worktree("my-repo", "feature/cleanup")
        .expect("worktree should exist");

    assert!(worktree.path().join("keep.txt").exists());
    assert!(!worktree.path().join("delete.txt").exists());
}

#[test]
fn test_worktree_git_helper() {
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
                    w.commit("Test commit", |c| {
                        c.file("test.txt", "test");
                    });
                });
            });
        });
    })
    .create();

    let worktree = env
        .find_worktree("my-repo", "feature/test")
        .expect("worktree should exist");

    let log = worktree.git(&["log", "--oneline"]);
    assert!(log.contains("Test commit"));
}
