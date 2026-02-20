use schemaforge::{format_for_tests, registry};
use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn golden_passes() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let passes_dir = manifest_dir.join("testdata/passes");

    let pass_dirs = read_sorted_dirs(&passes_dir).unwrap_or_else(|err| {
        panic!(
            "failed to read pass testdata dirs {}: {}",
            passes_dir.display(),
            err
        )
    });

    for pass_dir in pass_dirs {
        let pass_name = pass_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_else(|| {
                panic!("invalid utf-8 pass dir name: {}", pass_dir.display())
            });

        let spec = registry::find_pass(pass_name).unwrap_or_else(|| {
            panic!(
                "missing registry entry for pass '{}' (directory: {})",
                pass_name,
                pass_dir.display()
            )
        });

        let cases = read_sorted_files(&pass_dir).unwrap_or_else(|err| {
            panic!(
                "failed to read testcases for pass '{}' at {}: {}",
                pass_name,
                pass_dir.display(),
                err
            )
        });

        for input_path in cases {
            if !is_in_kdl(&input_path) {
                continue;
            }

            let base = strip_suffix(
                input_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_else(|| {
                        panic!(
                            "invalid utf-8 testcase filename for pass '{}': {}",
                            pass_name,
                            input_path.display()
                        )
                    }),
                ".in.kdl",
            )
            .unwrap_or_else(|| {
                panic!(
                    "input filename did not end with .in.kdl for pass '{}': {}",
                    pass_name,
                    input_path.display()
                )
            });

            let out_path = pass_dir.join(format!("{}.out.kdl", base));
            let err_path = pass_dir.join(format!("{}.err.txt", base));
            let has_out = out_path.exists();
            let has_err = err_path.exists();

            if has_out == has_err {
                panic!(
                    "pass '{}' case '{}' must have exactly one of {} or {}",
                    pass_name,
                    base,
                    out_path.display(),
                    err_path.display()
                );
            }

            let input =
                fs::read_to_string(&input_path).unwrap_or_else(|read_err| {
                    panic!(
                        "failed reading pass '{}' case '{}' input {}: {}",
                        pass_name,
                        base,
                        input_path.display(),
                        read_err
                    )
                });

            let result = (spec.run)(&input);

            if has_out {
                let expected = fs::read_to_string(&out_path).unwrap_or_else(|read_err| {
                    panic!(
                        "failed reading pass '{}' case '{}' expected output {}: {}",
                        pass_name,
                        base,
                        out_path.display(),
                        read_err
                    )
                });

                let actual = result.unwrap_or_else(|err| {
                    panic!(
                        "pass '{}' case '{}' expected success but failed:\n{}",
                        pass_name,
                        base,
                        format_for_tests(&err)
                    )
                });

                if actual != expected {
                    panic!(
                        "pass '{}' case '{}' output mismatch\n--- expected ---\n{}\n--- actual ---\n{}",
                        pass_name,
                        base,
                        expected,
                        actual
                    );
                }
            } else {
                let expected = fs::read_to_string(&err_path).unwrap_or_else(|read_err| {
                    panic!(
                        "failed reading pass '{}' case '{}' expected error {}: {}",
                        pass_name,
                        base,
                        err_path.display(),
                        read_err
                    )
                });

                let actual = result
                    .map(|output| {
                        panic!(
                            "pass '{}' case '{}' expected failure but succeeded with output:\n{}",
                            pass_name, base, output
                        )
                    })
                    .unwrap_err();

                let actual_text = format_for_tests(&actual);
                if actual_text != expected {
                    panic!(
                        "pass '{}' case '{}' error mismatch\n--- expected ---\n{}\n--- actual ---\n{}",
                        pass_name,
                        base,
                        expected,
                        actual_text
                    );
                }
            }
        }
    }
}

fn read_sorted_dirs(path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            entries.push(entry_path);
        }
    }
    entries.sort();
    Ok(entries)
}

fn read_sorted_files(path: &Path) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_file() {
            entries.push(entry_path);
        }
    }
    entries.sort();
    Ok(entries)
}

fn is_in_kdl(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.ends_with(".in.kdl"))
        .unwrap_or(false)
}

fn strip_suffix<'a>(value: &'a str, suffix: &str) -> Option<&'a str> {
    value.strip_suffix(suffix)
}
