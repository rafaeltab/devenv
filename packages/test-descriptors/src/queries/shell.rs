use std::process::ExitStatus;

/// Output from running a shell command
#[derive(Debug)]
pub struct ShellOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: ExitStatus,
}

impl ShellOutput {
    /// Check if the command succeeded (exit code 0)
    pub fn success(&self) -> bool {
        self.status.success()
    }

    /// Assert the command succeeded, panic with details if not
    pub fn assert_success(&self) -> &Self {
        if !self.success() {
            panic!(
                "Command failed with exit code {:?}\nstdout: {}\nstderr: {}",
                self.status.code(),
                self.stdout,
                self.stderr
            );
        }
        self
    }

    /// Assert the command failed, panic if it succeeded
    pub fn assert_failure(&self) -> &Self {
        if self.success() {
            panic!(
                "Expected command to fail but it succeeded\nstdout: {}\nstderr: {}",
                self.stdout, self.stderr
            );
        }
        self
    }

    /// Assert stdout contains the expected string
    pub fn assert_stdout_contains(&self, expected: &str) -> &Self {
        if !self.stdout.contains(expected) {
            panic!(
                "Expected stdout to contain {:?}\nActual stdout: {}",
                expected, self.stdout
            );
        }
        self
    }

    /// Assert stderr contains the expected string
    pub fn assert_stderr_contains(&self, expected: &str) -> &Self {
        if !self.stderr.contains(expected) {
            panic!(
                "Expected stderr to contain {:?}\nActual stderr: {}",
                expected, self.stderr
            );
        }
        self
    }

    /// Assert stdout equals the expected string (trimmed)
    pub fn assert_stdout_eq(&self, expected: &str) -> &Self {
        let actual = self.stdout.trim();
        if actual != expected {
            panic!(
                "Expected stdout to equal {:?}\nActual stdout: {:?}",
                expected, actual
            );
        }
        self
    }

    /// Get the exit code (if available)
    pub fn exit_code(&self) -> Option<i32> {
        self.status.code()
    }
}
