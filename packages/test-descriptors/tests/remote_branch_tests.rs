use std::process::Command;
use test_descriptors::TestEnvironment;

#[test]
fn test_remote_branch_with_single_commit() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("test-repo", |g| {
                g.remote("origin", |r| {
                    r.branch("feature", |b| {
                        b.commit("Add feature", |c| {
                            c.file("feature.txt", "feature content");
                        });
                    });
                });
            });
        });
    })
    .create();

    let repo_path = env.root_path().join("test-repo");

    Command::new("git")
        .args(["fetch", "origin"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let remote_refs = repo_path.join(".git").join("refs").join("remotes").join("origin");
    assert!(remote_refs.join("feature").exists());
}

#[test]
fn test_remote_branch_with_multiple_commits() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("test-repo", |g| {
                g.remote("origin", |r| {
                    r.branch("feature", |b| {
                        b.commit("First commit", |c| {
                            c.file("file1.txt", "content 1");
                        });
                        b.commit("Second commit", |c| {
                            c.file("file2.txt", "content 2");
                        });
                        b.commit("Third commit", |c| {
                            c.file("file3.txt", "content 3");
                        });
                    });
                });
            });
        })
    })
    .create();

    let repo_path = env.root_path().join("test-repo");

    Command::new("git")
        .args(["fetch", "origin"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let output = Command::new("git")
        .args(["rev-list", "--count", "origin/feature"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    let count = String::from_utf8_lossy(&output.stdout).trim().parse::<i32>().unwrap();
    assert_eq!(count, 4);
}

#[test]
fn test_remote_branch_files_exist_in_remote() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("test-repo", |g| {
                g.remote("origin", |r| {
                    r.branch("feature", |b| {
                        b.commit("Add files", |c| {
                            c.file("subdir/file.txt", "nested content");
                            c.file("root.txt", "root content");
                        });
                    });
                });
            });
        })
    })
    .create();

    let remote_path = {
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(&env.root_path().join("test-repo"))
            .output()
            .unwrap();
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    };

    let temp_clone = env.root_path().join("temp-clone");
    std::fs::create_dir_all(&temp_clone).unwrap();

    Command::new("git")
        .args(["clone", &remote_path, &temp_clone.to_string_lossy()])
        .current_dir(&env.root_path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["checkout", "feature"])
        .current_dir(&temp_clone)
        .output()
        .unwrap();

    assert!(temp_clone.join("subdir/file.txt").exists());
    assert!(temp_clone.join("root.txt").exists());

    std::fs::remove_dir_all(&temp_clone).ok();
}

#[test]
fn test_multiple_remote_branches() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("test-repo", |g| {
                g.remote("origin", |r| {
                    r.branch("production", |b| {
                        b.commit("Prod init", |c| {
                            c.file("prod.txt", "production");
                        });
                    });
                    r.branch("develop", |b| {
                        b.commit("Develop init", |c| {
                            c.file("dev.txt", "develop");
                        });
                    });
                    r.branch("feature/x", |b| {
                        b.commit("Feature", |c| {
                            c.file("feature.txt", "feature");
                        });
                    });
                });
            });
        })
    })
    .create();

    let repo_path = env.root_path().join("test-repo");

    Command::new("git")
        .args(["fetch", "origin"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let refs_path = repo_path.join(".git").join("refs").join("remotes").join("origin");
    assert!(refs_path.join("production").exists());
    assert!(refs_path.join("develop").exists());
    assert!(refs_path.join("feature").join("x").exists());
}

#[test]
fn test_remote_branch_inherits_from_parent_branch() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("test-repo", |g| {
                g.remote("origin", |r| {
                    r.branch_from("feature", "main", |b| {
                        b.commit("Feature work", |c| {
                            c.file("feature.txt", "feature");
                        });
                    });
                });
            });
        })
    })
    .create();

    let repo_path = env.root_path().join("test-repo");

    Command::new("git")
        .args(["fetch", "origin"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let output = Command::new("git")
        .args(["rev-list", "--count", "main"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    let main_count: i32 = String::from_utf8_lossy(&output.stdout).trim().parse().unwrap();

    let output = Command::new("git")
        .args(["rev-list", "--count", "origin/feature"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    let feature_count: i32 = String::from_utf8_lossy(&output.stdout).trim().parse().unwrap();

    assert_eq!(feature_count, main_count + 1);
}

#[test]
fn test_empty_remote_branch() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("test-repo", |g| {
                g.remote("origin", |r| {
                    r.branch("empty", |_| {});
                });
            });
        })
    })
    .create();

    let repo_path = env.root_path().join("test-repo");

    Command::new("git")
        .args(["fetch", "origin"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let remote_path = repo_path.join(".git/refs/remotes/origin/empty");
    assert!(remote_path.exists());
}

#[test]
fn test_remote_branch_with_file_deletions() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("test-repo", |g| {
                g.remote("origin", |r| {
                    r.branch("cleanup", |b| {
                        b.commit("Add file", |c| {
                            c.file("to-delete.txt", "temp");
                        });
                        b.commit("Delete file", |c| {
                            c.delete("to-delete.txt");
                        });
                    });
                });
            });
        })
    })
    .create();

    let remote_path = {
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(&env.root_path().join("test-repo"))
            .output()
            .unwrap();
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    };

    let temp_clone = env.root_path().join("temp-delete-check");
    std::fs::create_dir_all(&temp_clone).unwrap();

    Command::new("git")
        .args(["clone", &remote_path, &temp_clone.to_string_lossy()])
        .current_dir(&env.root_path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["checkout", "cleanup"])
        .current_dir(&temp_clone)
        .output()
        .unwrap();

    assert!(!temp_clone.join("to-delete.txt").exists());

    std::fs::remove_dir_all(&temp_clone).ok();
}

#[test]
fn test_remote_commit_message_matches() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("test-repo", |g| {
                g.remote("origin", |r| {
                    r.branch("feature", |b| {
                        b.commit("My special commit message", |c| {
                            c.file("file.txt", "content");
                        });
                    });
                });
            });
        })
    })
    .create();

    let remote_path = {
        let output = Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(&env.root_path().join("test-repo"))
            .output()
            .unwrap();
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    };

    let temp_clone = env.root_path().join("temp-checkout");
    std::fs::create_dir_all(&temp_clone).unwrap();

    Command::new("git")
        .args(["clone", &remote_path, &temp_clone.to_string_lossy()])
        .current_dir(&env.root_path())
        .output()
        .unwrap();

    Command::new("git")
        .args(["checkout", "feature"])
        .current_dir(&temp_clone)
        .output()
        .unwrap();

    let output = Command::new("git")
        .args(["log", "--format=%s", "-1"])
        .current_dir(&temp_clone)
        .output()
        .unwrap();
    let log = String::from_utf8_lossy(&output.stdout);
    assert!(log.contains("My special commit message"));

    std::fs::remove_dir_all(&temp_clone).ok();
}

#[test]
fn test_remote_and_local_branch_same_name() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("test-repo", |g| {
                g.remote("origin", |r| {
                    r.branch("feature", |b| {
                        b.commit("Remote commit", |c| {
                            c.file("remote.txt", "remote");
                        });
                    });
                });
                g.branch("feature", |b| {
                    b.commit("Local commit", |c| {
                        c.file("local.txt", "local");
                    });
                });
            });
        })
    })
    .create();

    let repo_path = env.root_path().join("test-repo");

    Command::new("git")
        .args(["fetch", "origin"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let refs = repo_path.join(".git/refs");
    let remotes = refs.join("remotes").join("origin").join("feature");
    let locals = refs.join("heads").join("feature");

    assert!(remotes.exists());
    assert!(locals.exists());
    assert_ne!(remotes, locals);
}

#[test]
fn test_fetch_remote_matches_declared_state() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("test-repo", |g| {
                g.remote("origin", |r| {
                    r.branch("feature", |b| {
                        b.commit("Feature commit", |c| {
                            c.file("feature.txt", "feature content");
                        });
                    });
                });
            });
        })
    })
    .create();

    let repo_path = env.root_path().join("test-repo");

    Command::new("git")
        .args(["fetch", "origin"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let output = Command::new("git")
        .args(["branch", "-r"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    let branches = String::from_utf8_lossy(&output.stdout);
    assert!(branches.contains("origin/feature"));
}

#[test]
fn test_remote_branch_name_with_slashes() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("test-repo", |g| {
                g.remote("origin", |r| {
                    r.branch("feature/my-feature", |b| {
                        b.commit("Add feature", |c| {
                            c.file("feature.txt", "content");
                        });
                    });
                    r.branch("release/v1.0", |b| {
                        b.commit("Release v1.0", |c| {
                            c.file("version.txt", "1.0.0");
                        });
                    });
                });
            });
        })
    })
    .create();

    let repo_path = env.root_path().join("test-repo");

    Command::new("git")
        .args(["fetch", "origin"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let refs = repo_path.join(".git/refs/remotes/origin");
    assert!(refs.join("feature").join("my-feature").exists());
    assert!(refs.join("release").join("v1.0").exists());
}

#[test]
fn test_remote_branch_from_initial_branch() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("test-repo", |g| {
                g.initial_branch("develop");
                g.remote("origin", |r| {
                    r.branch_from("feature", "main", |b| {
                        b.commit("Feature work", |c| {
                            c.file("feature.txt", "feature");
                        });
                    });
                });
            });
        })
    })
    .create();

    let repo_path = env.root_path().join("test-repo");

    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    let binding = String::from_utf8_lossy(&output.stdout);
    let current = binding.trim();
    assert_eq!(current, "develop");

    Command::new("git")
        .args(["fetch", "origin"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let output = Command::new("git")
        .args(["log", "--oneline", "origin/feature"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    let log = String::from_utf8_lossy(&output.stdout);
    assert!(log.contains("Feature work"));
}

#[test]
fn test_multiple_remotes_with_branches() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("test-repo", |g| {
                g.remote("origin", |r| {
                    r.branch("staging", |b| {
                        b.commit("Origin staging", |c| {
                            c.file("origin.txt", "origin");
                        });
                    });
                });
                g.remote("upstream", |r| {
                    r.branch("release", |b| {
                        b.commit("Upstream release", |c| {
                            c.file("upstream.txt", "upstream");
                        });
                    });
                });
            });
        })
    })
    .create();

    let repo_path = env.root_path().join("test-repo");

    Command::new("git")
        .args(["fetch", "--all"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let refs = repo_path.join(".git/refs/remotes");
    assert!(refs.join("origin/staging").exists());
    assert!(refs.join("upstream/release").exists());
}
