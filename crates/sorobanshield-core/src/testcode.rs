//! Detection of test-only code, shared by the engine and the rules.

use std::collections::HashSet;
use std::path::Path;
use syn::{Attribute, Item};

/// True for `#[cfg(test)]` and `#[cfg(any(test, ..))]`-style attributes.
pub fn is_cfg_test(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|a| match &a.meta {
        syn::Meta::List(l) => {
            a.path().is_ident("cfg")
                && l.tokens
                    .to_string()
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .any(|t| t == "test")
        }
        _ => false,
    })
}

/// Project-relative paths (`/`-separated) of files that are test-only: files
/// in a `tests/` directory (Cargo integration tests), files
/// with an inner `#![cfg(test)]`, files a parent declares with
/// `#[cfg(test)] mod name;`, and everything those files declare in turn.
///
/// Module files are located by the standard layout (`name.rs` or
/// `name/mod.rs` beside or below the declaring file). `#[path = ".."]`
/// attributes are not followed, so such a file is not recognised.
pub fn test_only_files(files: &[(String, &syn::File)]) -> HashSet<String> {
    let known: HashSet<&str> = files.iter().map(|(p, _)| p.as_str()).collect();
    let mut test_only: HashSet<String> = HashSet::new();
    for (path, ast) in files {
        if is_cfg_test(&ast.attrs) || in_tests_dir(path) {
            test_only.insert(path.clone());
        }
    }
    loop {
        let mut added = false;
        for (path, ast) in files {
            let from_test = test_only.contains(path);
            for item in &ast.items {
                let Item::Mod(m) = item else { continue };
                if m.content.is_some() || !(from_test || is_cfg_test(&m.attrs)) {
                    continue;
                }
                for candidate in module_files(path, &m.ident.to_string()) {
                    if known.contains(candidate.as_str()) && test_only.insert(candidate) {
                        added = true;
                    }
                }
            }
        }
        if !added {
            return test_only;
        }
    }
}

fn in_tests_dir(path: &str) -> bool {
    let mut parts: Vec<&str> = path.split('/').collect();
    parts.pop();
    parts.contains(&"tests")
}

fn module_files(declaring: &str, name: &str) -> Vec<String> {
    let p = Path::new(declaring);
    let dir = p.parent().unwrap_or(Path::new(""));
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let base = if matches!(stem, "lib" | "main" | "mod") {
        dir.to_path_buf()
    } else {
        dir.join(stem)
    };
    [
        base.join(format!("{name}.rs")),
        base.join(name).join("mod.rs"),
    ]
    .iter()
    .map(|c| c.to_string_lossy().replace('\\', "/"))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(spec: &[(&str, &str)]) -> Vec<(String, syn::File)> {
        spec.iter()
            .map(|(p, s)| (p.to_string(), syn::parse_file(s).unwrap()))
            .collect()
    }

    fn run(spec: &[(&str, &str)]) -> Vec<String> {
        let owned = files(spec);
        let refs: Vec<(String, &syn::File)> = owned.iter().map(|(p, f)| (p.clone(), f)).collect();
        let mut v: Vec<String> = test_only_files(&refs).into_iter().collect();
        v.sort();
        v
    }

    #[test]
    fn out_of_line_cfg_test_module_is_test_only() {
        let v = run(&[
            ("src/lib.rs", "#[cfg(test)] mod test;"),
            ("src/test.rs", "fn a() {}"),
        ]);
        assert_eq!(v, ["src/test.rs"]);
    }

    #[test]
    fn mod_rs_layout_and_nested_children_are_found() {
        let v = run(&[
            ("src/lib.rs", "#[cfg(test)] mod tests;"),
            ("src/tests/mod.rs", "mod helpers;"),
            ("src/tests/helpers.rs", ""),
        ]);
        assert_eq!(v, ["src/tests/helpers.rs", "src/tests/mod.rs"]);
    }

    #[test]
    fn inner_cfg_test_attribute_marks_the_file() {
        let v = run(&[
            ("src/lib.rs", "mod t;"),
            ("src/t.rs", "#![cfg(test)]\nfn a() {}"),
        ]);
        assert_eq!(v, ["src/t.rs"]);
    }

    #[test]
    fn files_in_a_tests_directory_are_test_only() {
        let v = run(&[
            ("src/lib.rs", ""),
            ("tests/it.rs", ""),
            ("crates/a/tests/nested/x.rs", ""),
            ("src/tests.rs", ""),
        ]);
        assert_eq!(v, ["crates/a/tests/nested/x.rs", "tests/it.rs"]);
    }

    #[test]
    fn non_test_modules_are_not_marked() {
        let v = run(&[("src/lib.rs", "mod storage;"), ("src/storage.rs", "")]);
        assert!(v.is_empty());
    }

    #[test]
    fn module_beside_a_non_root_file_lives_in_its_directory() {
        let v = run(&[("src/a.rs", "#[cfg(test)] mod t;"), ("src/a/t.rs", "")]);
        assert_eq!(v, ["src/a/t.rs"]);
    }
}
