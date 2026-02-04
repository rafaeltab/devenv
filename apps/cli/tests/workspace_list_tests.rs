mod common;

use crate::common::{
    rafaeltab_descriptors::RafaeltabDirMixin, rafaeltab_descriptors::RafaeltabRootMixin,
    CliTestRunner,
};
use test_descriptors::TestEnvironment;

#[test]
pub fn test_cli_integration() {
    let env = TestEnvironment::describe(|root| {
        root.rafaeltab_config(|c| {
            c.default_window("Neovim");
        });

        root.test_dir(|td| {
            td.dir("dotfiles", |d| {
                d.rafaeltab_workspace("dotfiles", "Dotfiles", |w| {
                    w.tag("dotfiles");
                    w.tag("lua");
                });
            });
            td.dir("notes", |d| {
                d.dir("coding_knowledge", |d| {
                    d.rafaeltab_workspace("coding_knowledge", "Notes", |w| {
                        w.tag("notes");
                        w.tag("markdown");
                    });
                });
            });
            td.dir("source", |d| {
                d.dir("rafaeltab", |d| {
                    d.rafaeltab_workspace("rafaeltab_cli", "Rafaeltab cli", |w| {
                        w.tag("rafaeltab");
                        w.tag("rust");
                    });
                });
                d.dir("code_analyzer", |d| {
                    d.rafaeltab_workspace("code_analyzer", "Code analyzer", |w| {
                        w.tag("rust");
                    });
                });
            });
        });
    })
    .create();

    let root_path = env.root_path();

    let (stdout, stderr, success) = CliTestRunner::new()
        .with_env(&env)
        .run(&["workspace", "list"]);

    assert!(
        success,
        "workspace list command should succeed.\nSTDOUT: {}\nSTDERR: {}",
        stdout, stderr
    );

    // Build expected output using the actual test directory paths
    let expected_dotfiles = format!(
        "Dotfiles (dotfiles): {} [\"dotfiles\", \"lua\"]",
        root_path.join("dotfiles").display()
    );
    let expected_notes = format!(
        "Notes (coding_knowledge): {} [\"notes\", \"markdown\"]",
        root_path.join("notes/coding_knowledge").display()
    );
    let expected_rafaeltab = format!(
        "Rafaeltab cli (rafaeltab_cli): {} [\"rafaeltab\", \"rust\"]",
        root_path.join("source/rafaeltab").display()
    );
    let expected_analyzer = format!(
        "Code analyzer (code_analyzer): {} [\"rust\"]",
        root_path.join("source/code_analyzer").display()
    );

    // Verify all workspaces are in the output
    assert!(
        stdout.contains(&expected_dotfiles),
        "Output should contain dotfiles workspace.\nExpected: {}\nGot: {}",
        expected_dotfiles,
        stdout
    );
    assert!(
        stdout.contains(&expected_notes),
        "Output should contain notes workspace.\nExpected: {}\nGot: {}",
        expected_notes,
        stdout
    );
    assert!(
        stdout.contains(&expected_rafaeltab),
        "Output should contain rafaeltab workspace.\nExpected: {}\nGot: {}",
        expected_rafaeltab,
        stdout
    );
    assert!(
        stdout.contains(&expected_analyzer),
        "Output should contain code analyzer workspace.\nExpected: {}\nGot: {}",
        expected_analyzer,
        stdout
    );

    // Verify we have exactly 4 workspaces (4 lines of output)
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        4,
        "Should have exactly 4 workspaces in output.\nGot {} lines: {:?}",
        lines.len(),
        lines
    );
}
