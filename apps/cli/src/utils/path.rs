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
