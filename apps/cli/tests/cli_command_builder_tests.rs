mod common;

use common::rafaeltab_descriptors::RafaeltabRootMixin;
use common::CliCommandBuilder;
use test_descriptors::TestEnvironment;

#[test]
fn builder_sets_binary_path() {
    let cmd = CliCommandBuilder::new().args(&["--version"]).build();

    assert_eq!(cmd.program(), env!("CARGO_BIN_EXE_rafaeltab"));
}

#[test]
fn builder_adds_config_flag() {
    let cmd = CliCommandBuilder::new()
        .with_config("/path/to/config.json")
        .args(&["tmux", "start"])
        .build();

    let args = cmd.build_args();
    assert_eq!(args[0], "--config");
    assert_eq!(args[1], "/path/to/config.json");
    assert_eq!(args[2], "tmux");
    assert_eq!(args[3], "start");
}

#[test]
fn builder_sets_tmux_socket_env() {
    let cmd = CliCommandBuilder::new()
        .with_tmux_socket("test-socket")
        .build();

    let envs = cmd.build_env();
    assert_eq!(
        envs.get("RAFAELTAB_TMUX_SOCKET"),
        Some(&"test-socket".to_string())
    );
}

#[test]
fn builder_with_env_extracts_from_environment() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| c.defaults());
    })
    .create();

    let cmd = CliCommandBuilder::new().with_env(&env).build();

    // Should have config path set
    let args = cmd.build_args();
    assert!(args.contains(&"--config".to_string()));

    // Should have tmux socket set
    let envs = cmd.build_env();
    assert!(envs.contains_key("RAFAELTAB_TMUX_SOCKET"));
}

#[test]
fn builder_with_cwd_sets_working_directory() {
    let cmd = CliCommandBuilder::new()
        .with_cwd("/some/path")
        .args(&["workspace", "list"])
        .build();

    assert_eq!(cmd.get_cwd(), Some(std::path::PathBuf::from("/some/path")));
}

#[test]
fn builder_with_env_var_adds_environment_variable() {
    let cmd = CliCommandBuilder::new()
        .with_env_var("DEBUG", "1")
        .with_env_var("RUST_LOG", "trace")
        .build();

    let envs = cmd.build_env();
    assert_eq!(envs.get("DEBUG"), Some(&"1".to_string()));
    assert_eq!(envs.get("RUST_LOG"), Some(&"trace".to_string()));
}

#[test]
fn builder_arg_adds_single_argument() {
    let cmd = CliCommandBuilder::new()
        .arg("workspace")
        .arg("list")
        .arg("--json")
        .build();

    let args = cmd.build_args();
    assert_eq!(args, vec!["workspace", "list", "--json"]);
}

#[test]
fn builder_args_and_arg_combine_correctly() {
    let cmd = CliCommandBuilder::new()
        .args(&["tmux", "start"])
        .arg("--session")
        .arg("my-session")
        .build();

    let args = cmd.build_args();
    assert_eq!(args, vec!["tmux", "start", "--session", "my-session"]);
}

#[test]
fn builder_explicit_config_overrides_with_env() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| c.defaults());
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_config("/override/config.json")
        .build();

    let args = cmd.build_args();
    let config_idx = args.iter().position(|a| a == "--config").unwrap();
    assert_eq!(args[config_idx + 1], "/override/config.json");
}

#[test]
fn builder_explicit_tmux_socket_overrides_with_env() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| c.defaults());
    })
    .create();

    let cmd = CliCommandBuilder::new()
        .with_env(&env)
        .with_tmux_socket("override-socket")
        .build();

    let envs = cmd.build_env();
    assert_eq!(
        envs.get("RAFAELTAB_TMUX_SOCKET"),
        Some(&"override-socket".to_string())
    );
}

#[test]
fn builder_default_pty_size() {
    let cmd = CliCommandBuilder::new().build();

    let (rows, cols) = cmd.get_pty_size();
    // Default size is 24x80 as per Command struct
    assert_eq!(rows, 24);
    assert_eq!(cols, 80);
}
