use test_descriptors::descriptor::{CommitDescriptor, FileChange};

#[test]
fn test_commit_descriptor_pushed_to_remote() {
    let commit = CommitDescriptor::new("Push me").pushed_to("origin");

    assert_eq!(commit.get_pushed_to(), Some("origin"));
}

#[test]
fn test_commit_descriptor_pushed_as_different_branch() {
    let commit = CommitDescriptor::new("Push me").pushed_as("origin", "remote-branch");

    assert_eq!(commit.get_pushed_to(), Some("origin"));
    assert_eq!(commit.get_pushed_as(), Some("remote-branch"));
}

#[test]
fn test_commit_descriptor_with_file_change() {
    let commit = CommitDescriptor::new("Add file").with_file("test.txt", "content");

    assert_eq!(commit.changes().len(), 1);
}

#[test]
fn test_commit_descriptor_with_multiple_files() {
    let commit = CommitDescriptor::new("Add files")
        .with_file("file1.txt", "content1")
        .with_file("file2.txt", "content2");

    assert_eq!(commit.changes().len(), 2);
}

#[test]
fn test_commit_descriptor_with_delete() {
    let commit = CommitDescriptor::new("Delete file").with_delete("old.txt");

    assert_eq!(commit.changes().len(), 1);
    match &commit.changes()[0] {
        FileChange::Delete { path } => assert_eq!(path, "old.txt"),
        _ => panic!("Expected Delete change"),
    }
}

#[test]
fn test_commit_descriptor_mixed_changes() {
    let commit = CommitDescriptor::new("Mixed changes")
        .with_file("new.txt", "new content")
        .with_delete("old.txt")
        .with_file("updated.txt", "updated content");

    assert_eq!(commit.changes().len(), 3);
}

#[test]
fn test_commit_descriptor_with_message() {
    let commit = CommitDescriptor::new("Initial commit");
    assert_eq!(commit.message(), "Initial commit");
}
