use test_descriptors::descriptor::{BranchDescriptor, CommitDescriptor};

#[test]
fn test_branch_descriptor_with_name() {
    let branch = BranchDescriptor::new("feature-branch");
    assert_eq!(branch.name(), "feature-branch");
}

#[test]
fn test_branch_descriptor_from_base() {
    let branch = BranchDescriptor::from("feature-branch", "main");
    assert_eq!(branch.name(), "feature-branch");
    assert_eq!(branch.base(), Some("main"));
}

#[test]
fn test_branch_descriptor_with_commits() {
    let branch = BranchDescriptor::new("feature")
        .with_commit(CommitDescriptor::new("First commit"))
        .with_commit(CommitDescriptor::new("Second commit"));

    assert_eq!(branch.commits().len(), 2);
    assert_eq!(branch.commits()[0].message(), "First commit");
    assert_eq!(branch.commits()[1].message(), "Second commit");
}

#[test]
fn test_branch_descriptor_no_base() {
    let branch = BranchDescriptor::new("standalone");
    assert_eq!(branch.base(), None);
}

#[test]
fn test_branch_descriptor_with_complex_commit() {
    let commit = CommitDescriptor::new("Add files")
        .with_file("test.txt", "content")
        .with_file("another.txt", "more content");

    let branch = BranchDescriptor::new("feature").with_commit(commit);

    assert_eq!(branch.commits().len(), 1);
    assert_eq!(branch.commits()[0].changes().len(), 2);
}
