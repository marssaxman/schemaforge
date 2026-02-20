use schemaforge::registry;
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn resolve_fixtures() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures_dir = manifest_dir.join("tests/fixtures/resolve");

    let spec = registry::find_pass("resolve").expect("pass should exist");

    let entries = fs::read_dir(&fixtures_dir).expect("read fixtures directory");
    for entry in entries {
        let entry = entry.expect("read fixture entry");
        let path = entry.path();
        if !is_input_fixture(&path) {
            continue;
        }

        let input = fs::read_to_string(&path).unwrap_or_else(|err| {
            panic!("failed to read {}: {}", path.display(), err)
        });

        let output = (spec.run)(&input).unwrap_or_else(|err| {
            panic!("pass failed for {}: {}", path.display(), err)
        });

        let expected_path = expected_output_path(&path);
        let expected =
            fs::read_to_string(&expected_path).unwrap_or_else(|err| {
                panic!(
                    "failed to read expected output {}: {}",
                    expected_path.display(),
                    err
                )
            });

        if output != expected {
            panic!(
                "fixture mismatch for {}\n--- expected ---\n{}\n--- actual ---\n{}",
                path.display(),
                expected,
                output
            );
        }
    }
}

fn is_input_fixture(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.ends_with(".in.ast.kdl"))
        .unwrap_or(false)
}

fn expected_output_path(input_path: &Path) -> PathBuf {
    let file_name = input_path
        .file_name()
        .and_then(|name| name.to_str())
        .expect("input fixture filename");

    let expected_name = file_name.replace(".in.ast.kdl", ".out.schema.kdl");
    input_path.with_file_name(expected_name)
}
