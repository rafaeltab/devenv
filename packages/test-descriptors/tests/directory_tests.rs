use test_descriptors::TestEnvironment;

#[test]
fn test_directory_descriptor_creates_directory() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("test-dir", |_d| {
                // Empty directory
            });
        });
    })
    .create();

    let expected_path = env.root_path().join("test-dir");
    assert!(expected_path.exists());
    assert!(expected_path.is_dir());
}

#[test]
fn test_directory_descriptor_path_resolution() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("my-directory", |_d| {});
        });
    })
    .create();

    // Use query API to check path
    let dir = env
        .find_dir("my-directory")
        .expect("directory should exist");
    assert_eq!(dir.path(), env.root_path().join("my-directory"));
}

#[test]
fn test_directory_descriptor_registered_in_context() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("registered-dir", |_d| {});
        });
    })
    .create();

    let dir = env.find_dir("registered-dir");
    assert!(dir.is_some());
    assert_eq!(dir.unwrap().path(), env.root_path().join("registered-dir"));
}

#[test]
fn test_directory_descriptor_already_exists_no_error() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("existing-dir", |_d| {});
            // Creating again should not error
            td.dir("existing-dir", |_d| {});
        });
    })
    .create();

    assert!(env.root_path().join("existing-dir").exists());
}

#[test]
fn test_directory_descriptor_nested() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("parent", |d| {
                d.dir("child", |_d| {});
            });
        });
    })
    .create();

    let expected_path = env.root_path().join("parent/child");
    assert!(expected_path.exists());
    assert!(expected_path.is_dir());
}

#[test]
fn test_directory_descriptor_deeply_nested() {
    let env = TestEnvironment::describe(|root| {
        root.test_dir(|td| {
            td.dir("parent", |d| {
                d.dir("child", |d| {
                    d.dir("grandchild", |_d| {});
                });
            });
        });
    })
    .create();

    let expected_path = env.root_path().join("parent/child/grandchild");
    assert!(expected_path.exists());
    assert!(expected_path.is_dir());
}
