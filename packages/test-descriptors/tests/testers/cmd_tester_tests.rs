//! 3.1 CmdTester Tests
//!
//! Tests specific to the CmdTester (subprocess execution outside tmux).

use test_descriptors::testers::{Command, CommandTester};
use test_descriptors::TestEnvironment;

/// $TMUX env var is not set.
#[test]
fn cmd_tester_runs_outside_tmux() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("sh").args(&["-c", "echo TMUX=$TMUX"]);
    let result = env.testers().cmd().run(&cmd);

    assert!(result.success);
    // TMUX should be empty or unset
    assert!(
        result.stdout.contains("TMUX=\n") || result.stdout.contains("TMUX=$"),
        "TMUX should not be set when running outside tmux"
    );
}

/// Command env vars are passed through.
#[test]
fn cmd_tester_inherits_env() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("workspace", |_d| {});
        });
    })
    .create();

    let cmd = Command::new("sh")
        .args(&["-c", "echo VAR1=$VAR1 VAR2=$VAR2"])
        .env("VAR1", "value1")
        .env("VAR2", "value2");
    let result = env.testers().cmd().run(&cmd);

    assert!(result.success);
    assert!(result.stdout.contains("VAR1=value1"));
    assert!(result.stdout.contains("VAR2=value2"));
}
