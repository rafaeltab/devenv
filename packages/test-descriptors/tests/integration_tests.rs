use std::fs;
use std::process::Command;
use test_descriptors::{
    BranchDescriptor, CommitDescriptor, DirectoryDescriptor, GitRepoDescriptor, RemoteDescriptor,
    TestEnvironment, TmuxSessionDescriptor, WindowDescriptor,
};

#[test]
fn test_full_environment_with_git_and_tmux() {
    let mut env = TestEnvironment::new();

    // Create a git repository with branches
    let feature_branch = BranchDescriptor::new("feature")
        .with_commit(CommitDescriptor::new("Add feature").with_file("feature.txt", "feature code"));

    let repo = GitRepoDescriptor::new("project")
        .with_branch(feature_branch)
        .with_remote(RemoteDescriptor::new("origin"));

    // Create a tmux session with windows
    let session = TmuxSessionDescriptor::new("dev")
        .with_window(WindowDescriptor::new("editor"))
        .with_window(WindowDescriptor::new("terminal"));

    // Create a workspace directory
    let workspace = DirectoryDescriptor::new("workspace");

    env.add_descriptor(workspace);
    env.add_descriptor(repo);
    env.add_descriptor(session);
    env.create().unwrap();

    // Verify everything was created
    assert!(env.root_path().join("workspace").exists());
    assert!(env.root_path().join("project/.git").exists());
    assert!(env.tmux().session_exists("dev"));

    // Verify git branches
    let output = Command::new("git")
        .args(&["branch", "--list"])
        .current_dir(env.root_path().join("project"))
        .output()
        .unwrap();
    let branches = String::from_utf8_lossy(&output.stdout);
    assert!(branches.contains("main"));
    assert!(branches.contains("feature"));

    // Verify tmux windows
    let output = env
        .tmux()
        .run_tmux(&["list-windows", "-t", "dev", "-F", "#{window_name}"])
        .unwrap();
    assert!(output.contains("editor"));
    assert!(output.contains("terminal"));
}

#[test]
fn test_multiple_branches_and_commits() {
    let mut env = TestEnvironment::new();

    // Create a repo with multiple branches
    let develop_branch = BranchDescriptor::new("develop")
        .with_commit(CommitDescriptor::new("Dev commit 1").with_file("dev1.txt", "dev 1"))
        .with_commit(CommitDescriptor::new("Dev commit 2").with_file("dev2.txt", "dev 2"));

    let feature_branch = BranchDescriptor::from("feature", "develop")
        .with_commit(CommitDescriptor::new("Feature commit").with_file("feature.txt", "feature"));

    let repo = GitRepoDescriptor::new("multi-branch")
        .with_branch(develop_branch)
        .with_branch(feature_branch);

    env.add_descriptor(repo);
    env.create().unwrap();

    let repo_path = env.root_path().join("multi-branch");

    // Verify all branches exist
    let output = Command::new("git")
        .args(&["branch", "--list"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    let branches = String::from_utf8_lossy(&output.stdout);
    assert!(branches.contains("main"));
    assert!(branches.contains("develop"));
    assert!(branches.contains("feature"));

    // Verify develop has 2 commits (plus initial)
    let output = Command::new("git")
        .args(&["rev-list", "--count", "develop"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    let count = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<i32>()
        .unwrap();
    assert_eq!(count, 3); // initial + 2 commits

    // Verify feature branch was created from develop
    Command::new("git")
        .args(&["checkout", "feature"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    assert!(repo_path.join("dev1.txt").exists());
    assert!(repo_path.join("dev2.txt").exists());
    assert!(repo_path.join("feature.txt").exists());
}

#[test]
fn test_git_push_to_local_remote() {
    let mut env = TestEnvironment::new();

    let repo = GitRepoDescriptor::new("push-test").with_remote(RemoteDescriptor::new("origin"));

    env.add_descriptor(repo);
    env.create().unwrap();

    let repo_path = env.root_path().join("push-test");

    // Make a commit and push it
    fs::write(repo_path.join("test.txt"), "test content").unwrap();

    Command::new("git")
        .args(&["add", "test.txt"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    Command::new("git")
        .args(&["commit", "-m", "Test commit"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let output = Command::new("git")
        .args(&["push", "origin", "main"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    assert!(output.status.success());

    // Verify the commit is in the remote
    let output = Command::new("git")
        .args(&["remote", "get-url", "origin"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    let remote_path = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let output = Command::new("git")
        .args(&["log", "--oneline"])
        .current_dir(&remote_path)
        .output()
        .unwrap();
    let log = String::from_utf8_lossy(&output.stdout);
    assert!(log.contains("Test commit"));
}

#[test]
fn test_nested_directory_structure() {
    let mut env = TestEnvironment::new();

    env.add_descriptor(DirectoryDescriptor::new("level1/level2/level3/level4"));
    env.create().unwrap();

    let deep_path = env.root_path().join("level1/level2/level3/level4");
    assert!(deep_path.exists());
    assert!(deep_path.is_dir());

    // Create a file in the deep directory
    let file_path = deep_path.join("deep.txt");
    fs::write(&file_path, "deep file").unwrap();
    assert!(file_path.exists());
}

#[test]
fn test_multiple_git_repos_in_same_environment() {
    let mut env = TestEnvironment::new();

    let repo1 = GitRepoDescriptor::new("repo1").with_branch(
        BranchDescriptor::new("feature1")
            .with_commit(CommitDescriptor::new("Repo1 feature").with_file("r1.txt", "r1")),
    );

    let repo2 = GitRepoDescriptor::new("repo2").with_branch(
        BranchDescriptor::new("feature2")
            .with_commit(CommitDescriptor::new("Repo2 feature").with_file("r2.txt", "r2")),
    );

    let repo3 = GitRepoDescriptor::new("nested/repo3");

    env.add_descriptor(repo1);
    env.add_descriptor(repo2);
    env.add_descriptor(repo3);
    env.create().unwrap();

    // Verify all repos exist
    assert!(env.root_path().join("repo1/.git").exists());
    assert!(env.root_path().join("repo2/.git").exists());
    assert!(env.root_path().join("nested/repo3/.git").exists());

    // Verify they don't interfere with each other
    let repo1_path = env.root_path().join("repo1");
    Command::new("git")
        .args(&["checkout", "feature1"])
        .current_dir(&repo1_path)
        .output()
        .unwrap();
    assert!(repo1_path.join("r1.txt").exists());
    assert!(!repo1_path.join("r2.txt").exists());

    let repo2_path = env.root_path().join("repo2");
    Command::new("git")
        .args(&["checkout", "feature2"])
        .current_dir(&repo2_path)
        .output()
        .unwrap();
    assert!(repo2_path.join("r2.txt").exists());
    assert!(!repo2_path.join("r1.txt").exists());
}

#[test]
fn test_environment_cleanup_removes_everything() {
    let root_path;
    let socket_name;
    let repo_path;

    {
        let mut env = TestEnvironment::new();
        root_path = env.root_path().to_path_buf();
        socket_name = env.tmux_socket().to_string();
        repo_path = root_path.join("cleanup-test");

        env.add_descriptor(GitRepoDescriptor::new("cleanup-test"));
        env.add_descriptor(DirectoryDescriptor::new("cleanup-dir"));
        env.add_descriptor(TmuxSessionDescriptor::new("cleanup-session"));
        env.create().unwrap();

        // Verify everything exists
        assert!(root_path.exists());
        assert!(repo_path.exists());
        assert!(root_path.join("cleanup-dir").exists());
        assert!(env.tmux().session_exists("cleanup-session"));
    } // env drops here

    // Verify everything is cleaned up
    assert!(!root_path.exists());
    assert!(!repo_path.exists());

    // Verify tmux session is gone
    let check = Command::new("tmux")
        .args(&["-L", &socket_name, "has-session", "-t", "cleanup-session"])
        .output()
        .unwrap();
    assert!(!check.status.success());
}

#[test]
fn test_commit_with_file_deletion() {
    let mut env = TestEnvironment::new();

    let branch = BranchDescriptor::new("deletions")
        .with_commit(
            CommitDescriptor::new("Add files")
                .with_file("keep.txt", "keep this")
                .with_file("delete.txt", "delete this"),
        )
        .with_commit(CommitDescriptor::new("Delete file").with_delete("delete.txt"));

    let repo = GitRepoDescriptor::new("deletion-test").with_branch(branch);

    env.add_descriptor(repo);
    env.create().unwrap();

    let repo_path = env.root_path().join("deletion-test");

    // Checkout the deletions branch
    Command::new("git")
        .args(&["checkout", "deletions"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    // Verify keep.txt exists and delete.txt doesn't
    assert!(repo_path.join("keep.txt").exists());
    assert!(!repo_path.join("delete.txt").exists());

    // Verify the deletion is in git history
    let output = Command::new("git")
        .args(&["log", "--oneline"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    let log = String::from_utf8_lossy(&output.stdout);
    assert!(log.contains("Delete file"));
}

#[test]
fn test_complex_scenario_workspace_with_multiple_features() {
    let mut env = TestEnvironment::new();

    // Create workspace structure
    env.add_descriptor(DirectoryDescriptor::new("workspaces"));
    env.add_descriptor(DirectoryDescriptor::new("workspaces/project-a"));
    env.add_descriptor(DirectoryDescriptor::new("workspaces/project-b"));

    // Create main repo in project-a
    let main_repo = GitRepoDescriptor::new("workspaces/project-a/main-repo")
        .with_remote(RemoteDescriptor::new("origin"))
        .with_branch(
            BranchDescriptor::new("develop").with_commit(
                CommitDescriptor::new("Setup project")
                    .with_file("package.json", "{\"name\": \"project-a\"}"),
            ),
        );

    // Create utility repo in project-b
    let util_repo = GitRepoDescriptor::new("workspaces/project-b/utils").with_branch(
        BranchDescriptor::new("feature/helpers").with_commit(
            CommitDescriptor::new("Add helpers").with_file("helpers.js", "module.exports = {}"),
        ),
    );

    // Create tmux session for each project
    let session_a = TmuxSessionDescriptor::new("project-a")
        .with_window(WindowDescriptor::new("editor"))
        .with_window(WindowDescriptor::new("server"));

    let session_b =
        TmuxSessionDescriptor::new("project-b").with_window(WindowDescriptor::new("editor"));

    env.add_descriptor(main_repo);
    env.add_descriptor(util_repo);
    env.add_descriptor(session_a);
    env.add_descriptor(session_b);
    env.create().unwrap();

    // Verify workspace structure
    assert!(env.root_path().join("workspaces/project-a").exists());
    assert!(env.root_path().join("workspaces/project-b").exists());

    // Verify repos
    assert!(env
        .root_path()
        .join("workspaces/project-a/main-repo/.git")
        .exists());
    assert!(env
        .root_path()
        .join("workspaces/project-b/utils/.git")
        .exists());

    // Verify tmux sessions
    assert!(env.tmux().session_exists("project-a"));
    assert!(env.tmux().session_exists("project-b"));

    // Verify session windows
    let output = env
        .tmux()
        .run_tmux(&["list-windows", "-t", "project-a"])
        .unwrap();
    assert!(output.contains("editor"));
    assert!(output.contains("server"));
}

#[test]
fn test_git_repo_with_initial_branch_master() {
    let mut env = TestEnvironment::new();

    let repo = GitRepoDescriptor::new("legacy-repo").with_initial_branch("master");

    env.add_descriptor(repo);
    env.create().unwrap();

    let repo_path = env.root_path().join("legacy-repo");

    let output = Command::new("git")
        .args(&["branch", "--show-current"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    assert_eq!(branch, "master");
}
