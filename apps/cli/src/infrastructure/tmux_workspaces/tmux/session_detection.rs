/// Get the name of the current tmux session.
/// Returns None if not running inside tmux.
pub fn get_current_tmux_session() -> Option<String> {
    // Check if we're in tmux by checking $TMUX environment variable
    if std::env::var("TMUX").is_err() {
        return None;
    }

    // Get session name using tmux display-message
    let output = std::process::Command::new("tmux")
        .args(["display-message", "-p", "#{session_name}"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let session_name = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();

    if session_name.is_empty() {
        return None;
    }

    Some(session_name)
}
