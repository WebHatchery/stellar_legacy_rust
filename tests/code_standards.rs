//! Structural checks that keep the local crate aligned with the shared RustGames
//! standards. These checks intentionally inspect the source tree rather than
//! relying on compiler accidents, so a future refactor cannot silently move a
//! rule out of view.

use std::fs;
use std::path::{Path, PathBuf};

fn rust_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_files(root, &mut files);
    files
}

fn collect_rust_files(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root)
        .unwrap_or_else(|error| panic!("could not inspect {}: {error}", root.display()))
    {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            if !matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some(".git" | "target")
            ) {
                collect_rust_files(&path, files);
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}

#[test]
fn source_files_stay_under_the_limit() {
    macroquad_toolkit::source_gate::assert_source_files_within_limit(
        env!("CARGO_MANIFEST_DIR"),
        &[],
    );
}

#[test]
fn production_modules_have_module_documentation() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let missing: Vec<_> = rust_files(&root)
        .into_iter()
        .filter(|path| {
            fs::read_to_string(path)
                .expect("Rust source")
                .lines()
                .all(|line| !line.trim_start().starts_with("//!"))
        })
        .collect();
    assert!(
        missing.is_empty(),
        "production modules need a //! header: {missing:?}"
    );
}

#[test]
fn test_code_and_test_only_helpers_live_under_tests() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let misplaced: Vec<_> = rust_files(&root)
        .into_iter()
        .filter(|path| {
            let source = fs::read_to_string(path).expect("Rust source");
            source.contains("#[test]") || source.contains("#[cfg(test)]")
        })
        .collect();
    assert!(
        misplaced.is_empty(),
        "test attributes must live in tests/: {misplaced:?}"
    );
    assert!(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/unit")
            .is_dir(),
        "the migrated unit suites must remain under tests/unit"
    );
}

#[test]
fn game_data_uses_the_toolkit_loader_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let data_sources = rust_files(&root.join("src/data"));
    let test_sources = rust_files(&root.join("tests/unit/data"));
    let direct_parse: Vec<_> = data_sources
        .into_iter()
        .chain(test_sources)
        .filter(|path| {
            let source = fs::read_to_string(path).expect("Rust source");
            source.contains("serde_json::from_str")
                || source.contains("serde_json::from_slice")
                || source.contains("serde_json::from_value")
        })
        .collect();
    assert!(
        direct_parse.is_empty(),
        "game-data modules must use macroquad-toolkit loading: {direct_parse:?}"
    );
}

#[test]
fn migrated_tests_record_why_large_feature_suites_are_distinct() {
    assert!(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/TEST_COVERAGE.md")
            .is_file(),
        "tests/TEST_COVERAGE.md must explain suites that exceed the five-case target"
    );
}
