use std::process::Command;
use test_descriptors::TestEnvironment;

#[test]
fn test_git_repo_descriptor_creates_repo() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("test-repo", |_g| {
                    // Empty repo with just defaults
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("workspace/test-repo");
    assert!(repo_path.exists());
    assert!(repo_path.join(".git").exists());
}

#[test]
fn test_git_repo_descriptor_has_initial_commit() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("test-repo", |_g| {});
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("workspace/test-repo");
    let output = Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let log = String::from_utf8_lossy(&output.stdout);
    assert!(log.contains("Initial commit"));
}

#[test]
fn test_git_repo_descriptor_on_main_branch() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("test-repo", |_g| {});
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("workspace/test-repo");
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(branch, "main");
}

#[test]
fn test_git_repo_descriptor_path_resolution() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("my-repo", |_g| {});
            });
        });
    })
    .create();

    let repo = env.find_git_repo("my-repo").expect("repo should exist");
    assert_eq!(repo.path(), env.root_path().join("workspace/my-repo"));
}

#[test]
fn test_git_repo_descriptor_registered_in_context() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("registered-repo", |_g| {});
            });
        });
    })
    .create();

    let repo = env.find_git_repo("registered-repo");
    assert!(repo.is_some());
    assert_eq!(
        repo.unwrap().path(),
        env.root_path().join("workspace/registered-repo")
    );
}

#[test]
fn test_git_repo_descriptor_has_git_config() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("test-repo", |_g| {});
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("workspace/test-repo");

    // Check user.name
    let output = Command::new("git")
        .args(["config", "user.name"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(name, "Test User");

    // Check user.email
    let output = Command::new("git")
        .args(["config", "user.email"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let email = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(email, "test@example.com");
}

#[test]
fn test_git_repo_descriptor_has_readme() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("test-repo", |_g| {});
            });
        });
    })
    .create();

    let readme_path = env.root_path().join("workspace/test-repo/README.md");
    assert!(readme_path.exists());

    let content = std::fs::read_to_string(&readme_path).unwrap();
    assert!(content.contains("test-repo"));
}

#[test]
fn test_git_repo_inside_nested_dir() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("parent", |d| {
                d.dir("child", |d| {
                    d.git("my-repo", |_g| {});
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("parent/child/my-repo");
    assert!(repo_path.exists());
    assert!(repo_path.join(".git").exists());
}

#[test]
fn test_git_repo_descriptor_with_branch() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("test-repo", |g| {
                    g.branch("feature", |b| {
                        b.commit("Feature commit", |c| {
                            c.file("feature.txt", "content");
                        });
                    });
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("workspace/test-repo");

    // Check that feature branch exists
    let output = Command::new("git")
        .args(["branch", "--list", "feature"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let branches = String::from_utf8_lossy(&output.stdout);
    assert!(branches.contains("feature"));

    // Check that feature.txt exists on feature branch
    Command::new("git")
        .args(["checkout", "feature"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    assert!(repo_path.join("feature.txt").exists());
}

#[test]
fn test_git_repo_descriptor_with_remote() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("test-repo", |g| {
                    g.remote("origin", |_| {});
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("workspace/test-repo");

    // Check that remote exists
    let output = Command::new("git")
        .args(["remote", "-v"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let remotes = String::from_utf8_lossy(&output.stdout);
    assert!(remotes.contains("origin"));
}

#[test]
fn test_git_repo_descriptor_custom_initial_branch() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.git("test-repo", |g| {
                    g.initial_branch("master");
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("workspace/test-repo");
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    assert!(output.status.success());
    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(branch, "master");
}
