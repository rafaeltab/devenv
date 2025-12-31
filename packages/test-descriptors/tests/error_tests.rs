use std::io;
use test_descriptors::descriptor::CreateError;

#[test]
fn test_create_error_display_formatting() {
    let err = CreateError::IoError("file not found".to_string());
    let display = format!("{}", err);
    assert!(display.contains("file not found"));

    let err = CreateError::GitError("not a git repo".to_string());
    let display = format!("{}", err);
    assert!(display.contains("not a git repo"));

    let err = CreateError::TmuxError("session not found".to_string());
    let display = format!("{}", err);
    assert!(display.contains("session not found"));
}

#[test]
fn test_create_error_from_io_error() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "test error");
    let create_err: CreateError = io_err.into();

    match create_err {
        CreateError::IoError(msg) => assert!(msg.contains("test error")),
        _ => panic!("Expected IoError variant"),
    }
}

#[test]
fn test_create_error_debug_output() {
    let err = CreateError::InvalidDescriptor("bad config".to_string());
    let debug = format!("{:?}", err);
    assert!(debug.contains("InvalidDescriptor"));
    assert!(debug.contains("bad config"));
}

#[test]
fn test_create_error_resource_not_found() {
    let err = CreateError::ResourceNotFound("repo-0".to_string());
    let display = format!("{}", err);
    assert!(display.contains("repo-0"));
}
