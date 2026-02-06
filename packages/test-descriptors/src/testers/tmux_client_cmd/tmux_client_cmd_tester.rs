use crate::descriptor::tmux_client::TmuxClientHandle;
use crate::descriptor::TmuxSocket;
use crate::testers::command::{Command, CommandResult};
use crate::testers::traits::CommandTester;
use uuid::Uuid;

/// Command tester that executes commands inside a tmux client via run-shell.
///
/// This tester runs commands inside a tmux session using `tmux run-shell`, which means
/// the `$TMUX` environment variable is automatically set by tmux. This is useful for
/// testing CLI commands that need to know they're running inside tmux.
#[derive(Debug)]
pub struct TmuxClientCmdTester<'a> {
    client: &'a TmuxClientHandle,
    socket: &'a TmuxSocket,
}

impl<'a> TmuxClientCmdTester<'a> {
    pub(crate) fn new(client: &'a TmuxClientHandle, socket: &'a TmuxSocket) -> Self {
        Self { client, socket }
    }
}

impl CommandTester for TmuxClientCmdTester<'_> {
    fn run(&self, cmd: &Command) -> CommandResult {
        // Generate unique separators to avoid conflicts with command output
        let separator = format!("===SEP_{}===", Uuid::new_v4().simple());
        let exit_marker = format!("===EXIT_{}===", Uuid::new_v4().simple());

        // Build the command setup (env vars only, cwd is handled by run-shell -c)
        let mut setup_parts = vec![];

        // Export environment variables
        for (key, value) in cmd.build_env() {
            // Escape single quotes in values
            let escaped_value = value.replace('\'', "'\\''");
            setup_parts.push(format!("export {}='{}'", key, escaped_value));
        }

        // Build the actual command with properly escaped arguments
        let program = cmd.program();
        let escaped_program = program.replace('\'', "'\\''");

        let args = cmd
            .build_args()
            .iter()
            .map(|a| {
                // Escape single quotes in arguments
                let escaped = a.replace('\'', "'\\''");
                format!("'{}'", escaped)
            })
            .collect::<Vec<_>>()
            .join(" ");

        let full_command = if args.is_empty() {
            format!("'{}'", escaped_program)
        } else {
            format!("'{}' {}", escaped_program, args)
        };

        // Build the setup portion (env vars only)
        let setup = if setup_parts.is_empty() {
            String::new()
        } else {
            setup_parts.join("; ") + "; "
        };

        // Build the wrapper script that captures stdout/stderr separately
        // The script:
        // 1. Creates temp files for stdout and stderr
        // 2. Runs the command redirecting output to temp files
        // 3. Captures the exit code
        // 4. Prints stdout, separator, stderr, exit marker with exit code
        // 5. Cleans up temp files
        let script = format!(
            r#"STDOUT_FILE=$(mktemp); STDERR_FILE=$(mktemp); {setup}{cmd} >"$STDOUT_FILE" 2>"$STDERR_FILE"; EXIT_CODE=$?; cat "$STDOUT_FILE"; printf '%s\n' '{sep}'; cat "$STDERR_FILE"; printf '%s%d\n' '{exit_marker}' "$EXIT_CODE"; rm -f "$STDOUT_FILE" "$STDERR_FILE""#,
            setup = setup,
            cmd = full_command,
            sep = separator,
            exit_marker = exit_marker
        );

        // Execute via tmux run-shell with -c for working directory
        // If cwd is specified, use -c to set the starting directory
        // This ensures std::env::current_dir() returns the correct path
        let _ = self.client; // Ensure client exists
        let output = if let Some(cwd) = cmd.get_cwd() {
            let cwd_str = cwd.to_string_lossy();
            self.socket
                .run_tmux(&["run-shell", "-c", &cwd_str, &script])
        } else {
            self.socket.run_tmux(&["run-shell", &script])
        };

        match output {
            Ok(raw_output) => self.parse_output(&raw_output, &separator, &exit_marker),
            Err(e) => CommandResult {
                stdout: String::new(),
                stderr: format!("Failed to run command in tmux: {}", e),
                exit_code: -1,
                success: false,
            },
        }
    }
}

impl TmuxClientCmdTester<'_> {
    /// Parse the raw output from tmux run-shell to extract stdout, stderr, and exit code.
    fn parse_output(&self, raw: &str, separator: &str, exit_marker: &str) -> CommandResult {
        // Split by separator to get stdout and the rest
        let parts: Vec<&str> = raw.splitn(2, separator).collect();

        let stdout = parts
            .first()
            .map(|s| s.trim_end_matches('\n').to_string())
            .unwrap_or_default();

        if parts.len() < 2 {
            // No separator found - something went wrong
            return CommandResult {
                stdout,
                stderr: String::new(),
                exit_code: -1,
                success: false,
            };
        }

        let rest = parts[1];

        // Find exit marker and extract exit code
        if let Some(exit_pos) = rest.find(exit_marker) {
            let stderr = rest[..exit_pos]
                .trim_start_matches('\n')
                .trim_end_matches('\n')
                .to_string();
            let exit_str = &rest[exit_pos + exit_marker.len()..];
            let exit_code: i32 = exit_str.trim().parse().unwrap_or(-1);

            CommandResult {
                stdout,
                stderr,
                exit_code,
                success: exit_code == 0,
            }
        } else {
            // No exit marker found - something went wrong
            CommandResult {
                stdout,
                stderr: rest.to_string(),
                exit_code: -1,
                success: false,
            }
        }
    }
}
