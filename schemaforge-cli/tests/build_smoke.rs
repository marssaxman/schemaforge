use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn build_generates_crate_files() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root =
        manifest_dir.parent().expect("workspace root").to_path_buf();

    let fixture =
        workspace_root.join("schemaforge/tests/fixtures/spike/spike.in.kdl");

    let output_dir = workspace_root.join("target/schemaforge-out/spike");
    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).expect("cleanup old output dir");
    }

    let binary = env!("CARGO_BIN_EXE_schemaforge-cli");
    let status = Command::new(binary)
        .current_dir(&workspace_root)
        .arg("build")
        .arg(&fixture)
        .status()
        .expect("run schemaforge-cli build");
    assert!(status.success());

    assert!(output_dir.join("Cargo.toml").exists());
    assert!(output_dir.join("src/lib.rs").exists());
    assert!(output_dir.join("src/main.rs").exists());
}
