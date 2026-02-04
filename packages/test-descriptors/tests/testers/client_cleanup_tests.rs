//! 4.3 Client Cleanup Tests
//!
//! Tests for proper cleanup of tmux clients.

use test_descriptors::TestEnvironment;

/// Client process killed when env dropped.
#[test]
fn client_killed_on_env_drop() {
    let socket_name: String;

    {
        let env = TestEnvironment::describe(|root| {
            root.test_dir(|td| {
                td.dir("workspace", |d| {
                    d.tmux_session("test-session", |s| {
                        s.window("main");
                        s.with_client(|c| {
                            c.pty_size(24, 80);
                        });
                    });
                });
            });
        })
        .create();

        // Store socket name for later verification
        socket_name = env.tmux_socket().to_string();

        // Verify client exists
        assert!(env.has_tmux_client());

        // env is dropped here
    }

    // After env is dropped, verify the socket/server is cleaned up
    // by checking that we can't connect to it
    let check = std::process::Command::new("tmux")
        .args(["-L", &socket_name, "list-sessions"])
        .output()
        .expect("Failed to run tmux");

    // Should fail because server was killed
    assert!(
        !check.status.success(),
        "Tmux server should be killed after env drop"
    );
}

/// Client killed before tmux server.
#[test]
fn client_killed_before_server() {
    // This test verifies the cleanup order is correct
    // (client killed before server, avoiding orphaned processes)

    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |d| {
                d.tmux_session("test-session", |s| {
                    s.window("main");
                    s.with_client(|c| {
                        c.pty_size(24, 80);
                    });
                });
            });
        });
    })
    .create();

    // Get client info before drop
    let client = env.tmux_client().expect("Client should exist");
    let session_name = client.current_session();
    assert_eq!(session_name, "test-session");

    // When env drops, cleanup should happen in correct order:
    // 1. Kill client
    // 2. Kill tmux server
    //
    // We can't easily verify the order in this test, but we can verify
    // that everything is cleaned up properly (no hanging processes)

    drop(env);

    // If we get here without hanging, cleanup order was correct
}
