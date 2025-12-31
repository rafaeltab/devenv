use std::fs;
use tempfile::TempDir;
use test_descriptors::descriptor::{
    CreateContext, Descriptor, DirectoryDescriptor, PathDescriptor,
};

#[test]
fn test_directory_descriptor_creates_directory() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let dir = DirectoryDescriptor::new("test-dir");
    dir.create(&context).unwrap();

    let expected_path = temp.path().join("test-dir");
    assert!(expected_path.exists());
    assert!(expected_path.is_dir());
}

#[test]
fn test_directory_descriptor_path_resolution() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let dir = DirectoryDescriptor::new("my-directory");
    let path = dir.path(&context);

    assert_eq!(path, temp.path().join("my-directory"));
}

#[test]
fn test_directory_descriptor_registered_in_context() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let dir = DirectoryDescriptor::new("registered-dir");
    dir.create(&context).unwrap();

    let retrieved = context.get_resource("registered-dir");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), temp.path().join("registered-dir"));
}

#[test]
fn test_directory_descriptor_already_exists_no_error() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let dir_path = temp.path().join("existing-dir");
    fs::create_dir(&dir_path).unwrap();

    let dir = DirectoryDescriptor::new("existing-dir");
    let result = dir.create(&context);

    assert!(result.is_ok());
    assert!(dir_path.exists());
}

#[test]
fn test_directory_descriptor_nested_path() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    // Create parent directory first
    fs::create_dir(temp.path().join("parent")).unwrap();

    let dir = DirectoryDescriptor::new("parent/child");
    dir.create(&context).unwrap();

    let expected_path = temp.path().join("parent/child");
    assert!(expected_path.exists());
    assert!(expected_path.is_dir());
}

#[test]
fn test_directory_descriptor_creates_parent_directories() {
    let temp = TempDir::new().unwrap();
    let context = CreateContext::new(temp.path().to_path_buf());

    let dir = DirectoryDescriptor::new("parent/child/grandchild");
    dir.create(&context).unwrap();

    let expected_path = temp.path().join("parent/child/grandchild");
    assert!(expected_path.exists());
    assert!(expected_path.is_dir());
}
