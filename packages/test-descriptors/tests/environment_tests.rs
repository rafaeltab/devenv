use std::fs;
use test_descriptors::TestEnvironment;

#[test]
fn test_environment_creates_temp_dir() {
    let env = TestEnvironment::new();

    assert!(env.root_path().exists());
    assert!(env.root_path().is_dir());
}

#[test]
fn test_environment_creates_tmux_socket() {
    let env = TestEnvironment::new();

    assert!(!env.tmux_socket().is_empty());
}

#[test]
fn test_environment_with_directory() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("test-dir", |_| {});
        });
    })
    .create();

    let dir = env.find_dir("test-dir").expect("dir should exist");
    assert!(dir.path().exists());
    assert!(dir.path().is_dir());
}

#[test]
fn test_environment_with_git_repo() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("my-repo", |_| {});
        });
    })
    .create();

    let repo = env.find_git_repo("my-repo").expect("repo should exist");
    assert!(repo.exists());
}

#[test]
fn test_environment_with_tmux_session() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("dev-session", |s| {
                    s.window("main");
                });
            });
        });
    })
    .create();

    let session = env
        .find_tmux_session("dev-session")
        .expect("session should exist");
    assert!(session.exists());
}

#[test]
fn test_environment_with_multiple_descriptors() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_| {});
            td.git("repo1", |_| {});
            td.git("repo2", |_| {});
            td.dir("session-dir", |d| {
                d.tmux_session("session1", |s| {
                    s.window("main");
                });
            });
        });
    })
    .create();

    assert!(env.find_dir("workspace").is_some());
    assert!(env.find_git_repo("repo1").is_some());
    assert!(env.find_git_repo("repo2").is_some());
    assert!(env.find_tmux_session("session1").is_some());
}

#[test]
fn test_environment_with_complex_git_repo() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("complex-repo", |g| {
                g.remote("origin", |_| {});
                g.branch("feature", |b| {
                    b.commit("Add feature", |c| {
                        c.file("feature.txt", "content");
                    });
                });
            });
        });
    })
    .create();

    let repo = env
        .find_git_repo("complex-repo")
        .expect("repo should exist");
    assert!(repo.exists());
    assert!(repo.branches().contains(&"feature".to_string()));
}

#[test]
fn test_environment_with_tmux_windows() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("multi-window", |s| {
                    s.window("editor");
                    s.window("terminal");
                    s.window("server");
                });
            });
        });
    })
    .create();

    let session = env
        .find_tmux_session("multi-window")
        .expect("session should exist");
    assert!(session.exists());
    assert_eq!(session.window_count(), 3);
}

#[test]
fn test_environment_cleanup_on_drop() {
    let root_path;
    let socket_name;

    {
        let env = TestEnvironment::describe(|root| {
            root.test_dir(|td| {
                td.dir("test-dir", |d| {
                    d.tmux_session("test-session", |s| {
                        s.window("main");
                    });
                });
            });
        })
        .create();

        root_path = env.root_path().to_path_buf();
        socket_name = env.tmux_socket().to_string();

        assert!(root_path.exists());
        assert!(env.tmux().session_exists("test-session"));
    } // env is dropped here

    // Verify temp dir is cleaned up
    assert!(!root_path.exists());

    // Verify tmux sessions are cleaned up
    let check = std::process::Command::new("tmux")
        .args(["-L", &socket_name, "has-session", "-t", "test-session"])
        .output()
        .unwrap();
    assert!(!check.status.success());
}

#[test]
fn test_environment_get_context() {
    let env = TestEnvironment::new();
    let context = env.context();

    assert_eq!(context.root_path(), env.root_path());
    assert_eq!(context.tmux_socket().as_deref(), Some(env.tmux_socket()));
}

#[test]
fn test_environment_registry_tracking() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.git("tracked-repo", |_| {});
            td.dir("tracked-dir", |d| {
                d.tmux_session("tracked-session", |s| {
                    s.window("main");
                });
            });
        });
    })
    .create();

    let context = env.context();
    let registry = context.registry().borrow();

    // Check that resources are tracked
    assert!(registry.get_git_repo("tracked-repo").is_some());
    assert!(registry.get_tmux_session("tracked-session").is_some());
}

#[test]
fn test_environment_create_file_in_directory() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_| {});
        });
    })
    .create();

    // Create a file in the directory
    let dir = env.find_dir("workspace").expect("dir should exist");
    let file_path = dir.path().join("test.txt");
    fs::write(&file_path, "test content").unwrap();

    assert!(file_path.exists());
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "test content");
}
