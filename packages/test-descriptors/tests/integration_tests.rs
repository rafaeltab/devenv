use std::fs;
use std::process::Command;
use test_descriptors::TestEnvironment;

#[test]
fn test_full_environment_with_git_and_tmux() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_| {});
            td.git("project", |g| {
                g.remote("origin");
                g.branch("feature", |b| {
                    b.commit("Add feature", |c| {
                        c.file("feature.txt", "feature code");
                    });
                });
            });
            td.dir("session-dir", |d| {
                d.tmux_session("dev", |s| {
                    s.window("editor");
                    s.window("terminal");
                });
            });
        });
    })
    .create();

    // Verify everything was created
    assert!(env.find_dir("workspace").is_some());
    let repo = env.find_git_repo("project").expect("repo should exist");
    assert!(repo.exists());

    // Verify git branches
    assert!(repo.branches().contains(&"main".to_string()));
    assert!(repo.branches().contains(&"feature".to_string()));

    // Verify tmux session and windows
    let session = env.find_tmux_session("dev").expect("session should exist");
    assert!(session.exists());
    let windows = session.windows();
    assert!(windows.contains(&"editor".to_string()));
    assert!(windows.contains(&"terminal".to_string()));
}

#[test]
fn test_multiple_branches_and_commits() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("multi-branch", |g| {
                g.branch("develop", |b| {
                    b.commit("Dev commit 1", |c| {
                        c.file("dev1.txt", "dev 1");
                    });
                    b.commit("Dev commit 2", |c| {
                        c.file("dev2.txt", "dev 2");
                    });
                });
                g.branch_from("feature", "develop", |b| {
                    b.commit("Feature commit", |c| {
                        c.file("feature.txt", "feature");
                    });
                });
            });
        });
    })
    .create();

    let repo = env
        .find_git_repo("multi-branch")
        .expect("repo should exist");
    let repo_path = repo.path();

    // Verify all branches exist
    assert!(repo.branches().contains(&"main".to_string()));
    assert!(repo.branches().contains(&"develop".to_string()));
    assert!(repo.branches().contains(&"feature".to_string()));

    // Verify develop has 2 commits (plus initial)
    let output = Command::new("git")
        .args(["rev-list", "--count", "develop"])
        .current_dir(repo_path)
        .output()
        .unwrap();
    let count = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<i32>()
        .unwrap();
    assert_eq!(count, 3); // initial + 2 commits

    // Verify feature branch was created from develop
    Command::new("git")
        .args(["checkout", "feature"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    assert!(repo_path.join("dev1.txt").exists());
    assert!(repo_path.join("dev2.txt").exists());
    assert!(repo_path.join("feature.txt").exists());
}

#[test]
fn test_git_push_to_local_remote() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("push-test", |g| {
                g.remote("origin");
            });
        });
    })
    .create();

    let repo = env.find_git_repo("push-test").expect("repo should exist");
    let repo_path = repo.path();

    // Make a commit and push it
    fs::write(repo_path.join("test.txt"), "test content").unwrap();

    Command::new("git")
        .args(["add", "test.txt"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "Test commit"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    let output = Command::new("git")
        .args(["push", "origin", "main"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    assert!(output.status.success());

    // Verify the commit is in the remote
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(repo_path)
        .output()
        .unwrap();
    let remote_path = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let output = Command::new("git")
        .args(["log", "--oneline"])
        .current_dir(&remote_path)
        .output()
        .unwrap();
    let log = String::from_utf8_lossy(&output.stdout);
    assert!(log.contains("Test commit"));
}

#[test]
fn test_nested_directory_structure() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("level1", |d1| {
                d1.dir("level2", |d2| {
                    d2.dir("level3", |d3| {
                        d3.dir("level4", |_| {});
                    });
                });
            });
        });
    })
    .create();

    let deep_dir = env.find_dir("level4").expect("deep dir should exist");
    assert!(deep_dir.path().exists());
    assert!(deep_dir.path().is_dir());

    // Create a file in the deep directory
    let file_path = deep_dir.path().join("deep.txt");
    fs::write(&file_path, "deep file").unwrap();
    assert!(file_path.exists());
}

#[test]
fn test_multiple_git_repos_in_same_environment() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("repo1", |g| {
                g.branch("feature1", |b| {
                    b.commit("Repo1 feature", |c| {
                        c.file("r1.txt", "r1");
                    });
                });
            });
            td.git("repo2", |g| {
                g.branch("feature2", |b| {
                    b.commit("Repo2 feature", |c| {
                        c.file("r2.txt", "r2");
                    });
                });
            });
            td.dir("nested", |d| {
                d.git("repo3", |_| {});
            });
        });
    })
    .create();

    // Verify all repos exist
    let repo1 = env.find_git_repo("repo1").expect("repo1 should exist");
    let repo2 = env.find_git_repo("repo2").expect("repo2 should exist");
    let repo3 = env.find_git_repo("repo3").expect("repo3 should exist");

    assert!(repo1.exists());
    assert!(repo2.exists());
    assert!(repo3.exists());

    // Verify they don't interfere with each other
    let repo1_path = repo1.path();
    Command::new("git")
        .args(["checkout", "feature1"])
        .current_dir(repo1_path)
        .output()
        .unwrap();
    assert!(repo1_path.join("r1.txt").exists());
    assert!(!repo1_path.join("r2.txt").exists());

    let repo2_path = repo2.path();
    Command::new("git")
        .args(["checkout", "feature2"])
        .current_dir(repo2_path)
        .output()
        .unwrap();
    assert!(repo2_path.join("r2.txt").exists());
    assert!(!repo2_path.join("r1.txt").exists());
}

#[test]
fn test_environment_cleanup_removes_everything() {
    let root_path;
    let socket_name;
    let repo_path;

    {
        let env = TestEnvironment::describe(|root| {
            root.test_dir(|td| {
                td.git("cleanup-test", |_| {});
                td.dir("cleanup-dir", |d| {
                    d.tmux_session("cleanup-session", |s| {
                        s.window("main");
                    });
                });
            });
        })
        .create();

        root_path = env.root_path().to_path_buf();
        socket_name = env.tmux_socket().to_string();
        repo_path = env
            .find_git_repo("cleanup-test")
            .unwrap()
            .path()
            .to_path_buf();

        // Verify everything exists
        assert!(root_path.exists());
        assert!(repo_path.exists());
        assert!(env.find_dir("cleanup-dir").is_some());
        assert!(env.tmux().session_exists("cleanup-session"));
    } // env drops here

    // Verify everything is cleaned up
    assert!(!root_path.exists());
    assert!(!repo_path.exists());

    // Verify tmux session is gone
    let check = Command::new("tmux")
        .args(["-L", &socket_name, "has-session", "-t", "cleanup-session"])
        .output()
        .unwrap();
    assert!(!check.status.success());
}

#[test]
fn test_commit_with_file_deletion() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("deletion-test", |g| {
                g.branch("deletions", |b| {
                    b.commit("Add files", |c| {
                        c.file("keep.txt", "keep this");
                        c.file("delete.txt", "delete this");
                    });
                    b.commit("Delete file", |c| {
                        c.delete("delete.txt");
                    });
                });
            });
        });
    })
    .create();

    let repo = env
        .find_git_repo("deletion-test")
        .expect("repo should exist");
    let repo_path = repo.path();

    // Checkout the deletions branch
    Command::new("git")
        .args(["checkout", "deletions"])
        .current_dir(repo_path)
        .output()
        .unwrap();

    // Verify keep.txt exists and delete.txt doesn't
    assert!(repo_path.join("keep.txt").exists());
    assert!(!repo_path.join("delete.txt").exists());

    // Verify the deletion is in git history
    let log = repo.git(&["log", "--oneline"]);
    assert!(log.contains("Delete file"));
}

#[test]
fn test_complex_scenario_workspace_with_multiple_features() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspaces", |ws| {
                ws.dir("project-a", |pa| {
                    pa.git("main-repo", |g| {
                        g.remote("origin");
                        g.branch("develop", |b| {
                            b.commit("Setup project", |c| {
                                c.file("package.json", "{\"name\": \"project-a\"}");
                            });
                        });
                    });
                    pa.tmux_session("project-a", |s| {
                        s.window("editor");
                        s.window("server");
                    });
                });
                ws.dir("project-b", |pb| {
                    pb.git("utils", |g| {
                        g.branch("feature/helpers", |b| {
                            b.commit("Add helpers", |c| {
                                c.file("helpers.js", "module.exports = {}");
                            });
                        });
                    });
                    pb.tmux_session("project-b", |s| {
                        s.window("editor");
                    });
                });
            });
        });
    })
    .create();

    // Verify workspace structure
    assert!(env.find_dir("project-a").is_some());
    assert!(env.find_dir("project-b").is_some());

    // Verify repos
    let main_repo = env
        .find_git_repo("main-repo")
        .expect("main-repo should exist");
    let utils_repo = env.find_git_repo("utils").expect("utils should exist");
    assert!(main_repo.exists());
    assert!(utils_repo.exists());

    // Verify tmux sessions
    let session_a = env
        .find_tmux_session("project-a")
        .expect("project-a session should exist");
    let session_b = env
        .find_tmux_session("project-b")
        .expect("project-b session should exist");
    assert!(session_a.exists());
    assert!(session_b.exists());

    // Verify session windows
    let windows = session_a.windows();
    assert!(windows.contains(&"editor".to_string()));
    assert!(windows.contains(&"server".to_string()));
}

#[test]
fn test_git_repo_with_initial_branch_master() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("legacy-repo", |g| {
                g.initial_branch("master");
            });
        });
    })
    .create();

    let repo = env.find_git_repo("legacy-repo").expect("repo should exist");
    assert_eq!(repo.current_branch(), "master");
}
