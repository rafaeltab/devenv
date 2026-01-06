use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let test_programs_dir = PathBuf::from(&manifest_dir)
        .join("tests")
        .join("test_programs");

    // Compile standalone .rs files in test_programs
    compile_standalone_programs(&test_programs_dir);

    // Build the key_detector crate (uses crossterm for reliable key detection)
    build_key_detector_crate(&test_programs_dir);
}

fn compile_standalone_programs(test_programs_dir: &Path) {
    if let Ok(entries) = fs::read_dir(test_programs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let source_file = path.clone();
                let binary_name = path.file_stem().unwrap().to_str().unwrap();
                let binary_path = test_programs_dir.join(binary_name);

                println!("cargo:rerun-if-changed={}", source_file.display());

                let should_compile = if binary_path.exists() {
                    let source_modified =
                        fs::metadata(&source_file).and_then(|m| m.modified()).ok();
                    let binary_modified =
                        fs::metadata(&binary_path).and_then(|m| m.modified()).ok();

                    match (source_modified, binary_modified) {
                        (Some(src), Some(bin)) => src > bin,
                        _ => true,
                    }
                } else {
                    true
                };

                if should_compile {
                    println!("cargo:warning=Compiling test program: {}", binary_name);

                    let status = Command::new("rustc")
                        .arg(&source_file)
                        .arg("-o")
                        .arg(&binary_path)
                        .status()
                        .expect("Failed to compile test program");

                    if !status.success() {
                        panic!("Failed to compile test program: {}", binary_name);
                    }
                }
            }
        }
    }
}

fn build_key_detector_crate(test_programs_dir: &Path) {
    let key_detector_dir = test_programs_dir.join("key_detector_crate");
    let main_rs = key_detector_dir.join("main.rs");
    let cargo_toml = key_detector_dir.join("Cargo.toml");
    let binary_path = key_detector_dir.join("target/release/key_detector");

    // Tell cargo to rerun if sources change
    println!("cargo:rerun-if-changed={}", main_rs.display());
    println!("cargo:rerun-if-changed={}", cargo_toml.display());

    // Check if we need to rebuild
    let should_build = if binary_path.exists() {
        let main_modified = fs::metadata(&main_rs).and_then(|m| m.modified()).ok();
        let binary_modified = fs::metadata(&binary_path).and_then(|m| m.modified()).ok();

        match (main_modified, binary_modified) {
            (Some(src), Some(bin)) => src > bin,
            _ => true,
        }
    } else {
        true
    };

    if should_build {
        println!("cargo:warning=Building key_detector crate");

        let status = Command::new("cargo")
            .arg("build")
            .arg("--release")
            .current_dir(&key_detector_dir)
            .status()
            .expect("Failed to build key_detector crate");

        if !status.success() {
            panic!("Failed to build key_detector crate");
        }
    }
}
