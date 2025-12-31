use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Git error: {0}")]
    GitError(String),

    #[error("Tmux error: {0}")]
    TmuxError(String),

    #[error("Invalid descriptor: {0}")]
    InvalidDescriptor(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
}

impl From<io::Error> for CreateError {
    fn from(err: io::Error) -> Self {
        CreateError::IoError(err.to_string())
    }
}
