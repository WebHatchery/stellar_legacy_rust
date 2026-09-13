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

const FUNCTION_LINE_LIMIT: usize = 100;

fn code_without_literals_or_comments(line: &str, block_comment: &mut bool) -> String {
    let mut code = String::new();
    let mut index = 0;
    let bytes = line.as_bytes();
    while index < bytes.len() {
        if *block_comment {
            if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'/') {
                *block_comment = false;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'/') {
            break;
        }
        if bytes[index] == b'/' && bytes.get(index + 1) == Some(&b'*') {
            *block_comment = true;
            index += 2;
            continue;
        }
        if bytes[index] == b'"' {
            index += 1;
            while index < bytes.len() {
                if bytes[index] == b'\\' {
                    index += 2;
                } else if bytes[index] == b'"' {
                    index += 1;
                    break;
                } else {
                    index += 1;
                }
            }
            continue;
        }
        if bytes[index] == b'\'' && index + 2 < bytes.len() && bytes[index + 2] == b'\'' {
            index += 3;
            continue;
        }
        code.push(bytes[index] as char);
        index += 1;
    }
    code
}

fn function_line_spans(source: &str) -> Vec<(usize, usize)> {
    let lines: Vec<_> = source.lines().collect();
    let mut starts = Vec::new();
    let mut block_comment = false;
    for (line_number, line) in lines.iter().enumerate() {
        let code = code_without_literals_or_comments(line, &mut block_comment);
        if let Some(fn_index) = code.find("fn ") {
            let after_fn = &code[fn_index + 3..];
            if after_fn
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
                && after_fn.contains('(')
            {
                starts.push(line_number);
            }
        }
    }

    let mut spans = Vec::new();
    for start in starts {
        let mut depth = 0;
        let mut saw_body = false;
        let mut block_comment = false;
        for (offset, line) in lines[start..].iter().enumerate() {
            let code = code_without_literals_or_comments(line, &mut block_comment);
            depth += code.chars().filter(|character| *character == '{').count();
            depth -= code.chars().filter(|character| *character == '}').count();
            saw_body |= code.contains('{');
            if saw_body && depth == 0 {
                spans.push((start + 1, start + offset + 1));
                break;
            }
        }
    }
    spans
}

#[test]
fn source_files_stay_under_the_limit() {
    macroquad_toolkit::source_gate::assert_source_files_within_limit(
        env!("CARGO_MANIFEST_DIR"),
        &[],
    );
}

#[test]
fn production_functions_stay_under_the_limit() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let violations: Vec<_> = rust_files(&root)
        .into_iter()
        .flat_map(|path| {
            let display = path
                .strip_prefix(&root)
                .expect("production source under src")
                .display()
                .to_string();
            let spans = function_line_spans(&fs::read_to_string(&path).expect("Rust source"));
            spans.into_iter().filter_map(move |(start, end)| {
                let lines = end - start + 1;
                (lines > FUNCTION_LINE_LIMIT).then(|| format!("{display}:{start}-{end} ({lines})"))
            })
        })
        .collect();
    assert!(
        violations.is_empty(),
        "production functions exceed {FUNCTION_LINE_LIMIT} lines:\n{}",
        violations.join("\n")
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
