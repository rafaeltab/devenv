use shellexpand::tilde;
use std::path::Path;

/// Expand and canonicalize a path.
///
/// This function:
/// 1. Expands `~` to the home directory
/// 2. Canonicalizes the path to resolve symlinks (e.g., /var -> /private/var on macOS)
///
/// If canonicalization fails (e.g., path doesn't exist), it returns the expanded path.
/// This ensures existing paths are always canonicalized while preserving backward
/// compatibility for paths that don't exist yet.
pub fn expand_path(path: &str) -> String {
    // Step 1: Expand tilde to home directory
    let expanded = tilde(path).to_string();

    // Step 2: Try to canonicalize to resolve symlinks
    // This handles macOS /private/var -> /var and similar OS-specific path differences
    match Path::new(&expanded).canonicalize() {
        Ok(canonical) => canonical.to_string_lossy().to_string(),
        Err(_) => {
            // Path doesn't exist or can't be canonicalized - return expanded path
            // This maintains backward compatibility for paths that haven't been created yet
            expanded
        }
    }
}
