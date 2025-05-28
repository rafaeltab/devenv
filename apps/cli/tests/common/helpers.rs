use std::fs::File;
use std::io::Write;
use std::process::Command;

use shellexpand::{full, tilde};

pub fn expand_path(path: &str) -> String {
    let res = match full(path) {
        Ok(val) => val,
        Err(err) => {
            eprintln!(
                "Encountered error while expanding path, defaulting to only tilde replacement: {}",
                err
            );
            tilde(path)
        }
    };
    res.to_string()
}

pub fn setup_test_file(content: &str) {
    let mut file = File::create(get_path()).expect("Unable to create file");
    file.write_all(content.as_bytes())
        .expect("Unable to write data");
}

pub fn run_cli_with_stdin(args: &[&str], input: &str) -> (String, String) {
    let mut command = Command::new("target/debug/rafaeltab")
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to execute process");

    {
        let stdin = command.stdin.as_mut().expect("failed to open stdin");
        stdin
            .write_all(input.as_bytes())
            .expect("failed to write to stdin");
    }

    let output = command.wait_with_output().expect("failed to read stdout");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (stdout, stderr)
}

pub fn verify_output(expected: &str, actual: &str) {
    assert_eq!(expected, actual, "Output did not match");
}

fn get_path() -> String {
    expand_path("~/.rafaeltab.json")
}
