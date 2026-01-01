use std::process::Command;
use test_descriptors::TestEnvironment;

#[test]
fn test_branch_with_commit() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("test-repo", |g| {
                    g.branch("feature-branch", |b| {
                        b.commit("Feature commit", |c| {
                            c.file("feature.txt", "feature content");
                        });
                    });
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("workspace/test-repo");

    // Check branch exists
    let output = Command::new("git")
        .args(["branch", "--list", "feature-branch"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let branches = String::from_utf8_lossy(&output.stdout);
    assert!(branches.contains("feature-branch"));
}

#[test]
fn test_branch_from_base() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("test-repo", |g| {
                    // First create develop branch with a commit
                    g.branch("develop", |b| {
                        b.commit("Develop commit", |c| {
                            c.file("develop.txt", "develop content");
                        });
                    });
                    // Then create feature branch from develop
                    g.branch_from("feature", "develop", |b| {
                        b.commit("Feature commit", |c| {
                            c.file("feature.txt", "feature content");
                        });
                    });
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("workspace/test-repo");

    // Switch to feature and check develop.txt exists (inherited from develop)
    Command::new("git")
        .args(["checkout", "feature"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    assert!(
        repo_path.join("develop.txt").exists(),
        "feature branch should have develop.txt from base"
    );
    assert!(
        repo_path.join("feature.txt").exists(),
        "feature branch should have its own feature.txt"
    );
}

#[test]
fn test_branch_with_multiple_commits() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("test-repo", |g| {
                    g.branch("feature", |b| {
                        b.commit("First commit", |c| {
                            c.file("first.txt", "first");
                        });
                        b.commit("Second commit", |c| {
                            c.file("second.txt", "second");
                        });
                    });
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("workspace/test-repo");

    // Checkout feature and verify both files exist
    Command::new("git")
        .args(["checkout", "feature"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    assert!(repo_path.join("first.txt").exists());
    assert!(repo_path.join("second.txt").exists());

    // Verify commit count (initial + 2 feature commits)
    let output = Command::new("git")
        .args(["rev-list", "--count", "feature"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let count: i32 = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .unwrap();
    assert_eq!(count, 3); // initial + 2 commits
}

#[test]
fn test_multiple_branches() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("test-repo", |g| {
                    g.branch("feature-a", |b| {
                        b.commit("Feature A", |c| {
                            c.file("a.txt", "a");
                        });
                    });
                    g.branch("feature-b", |b| {
                        b.commit("Feature B", |c| {
                            c.file("b.txt", "b");
                        });
                    });
                });
            });
        });
    })
    .create();

    let repo = env.find_git_repo("test-repo").expect("repo should exist");
    let branches = repo.branches();

    assert!(branches.contains(&"main".to_string()));
    assert!(branches.contains(&"feature-a".to_string()));
    assert!(branches.contains(&"feature-b".to_string()));
}

#[test]
fn test_commit_with_delete() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("test-repo", |g| {
                    g.branch("cleanup", |b| {
                        b.commit("Add file", |c| {
                            c.file("temp.txt", "temporary");
                        });
                        b.commit("Remove file", |c| {
                            c.delete("temp.txt");
                        });
                    });
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("workspace/test-repo");

    Command::new("git")
        .args(["checkout", "cleanup"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    // File should be deleted
    assert!(!repo_path.join("temp.txt").exists());
}
