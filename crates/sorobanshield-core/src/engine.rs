use crate::finding::Finding;
use crate::project::{discover_projects, Project, ProjectError};
use crate::rule::{relative_display, ManifestUnit, ProjectUnit, Rule, SourceUnit};
use crate::suppress::{self, Suppression, UnusedSuppression};
use crate::testcode;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScanError {
    pub file: String,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct ScanOutcome {
    pub files_scanned: usize,
    pub findings: Vec<Finding>,
    /// Files that could not be read or parsed. A scan with errors is not a clean scan.
    pub errors: Vec<ScanError>,
    /// Findings hidden by `sorobanshield-ignore` comments.
    pub suppressed: Vec<Suppression>,
    /// Rule IDs in suppression comments that matched no finding.
    pub unused_suppressions: Vec<UnusedSuppression>,
}

pub fn scan_project(project: &Project, rules: &[Box<dyn Rule>]) -> ScanOutcome {
    let mut out = ScanOutcome::default();
    let mut parsed: Vec<(String, syn::File)> = Vec::new();
    let mut directives = Vec::new();
    for path in &project.sources {
        let rel = relative_display(&project.root, path);
        match parse(path) {
            Ok((ast, text)) => {
                directives.extend(suppress::parse_directives(&rel, &text));
                parsed.push((rel, ast));
            }
            Err(message) => out.errors.push(ScanError { file: rel, message }),
        }
    }
    out.files_scanned = parsed.len();
    let units: Vec<SourceUnit<'_>> = parsed
        .iter()
        .map(|(path, ast)| SourceUnit { path, ast })
        .collect();
    // Test-only files stay in `units` so project-level rules (SS-005) can read
    // test code, but per-file rules must not report on them.
    let test_only = testcode::test_only_files(
        &parsed
            .iter()
            .map(|(p, a)| (p.clone(), a))
            .collect::<Vec<_>>(),
    );
    for unit in units.iter().filter(|u| !test_only.contains(u.path)) {
        for rule in rules {
            out.findings.extend(rule.check(unit));
        }
    }
    let project_unit = ProjectUnit {
        files: &units,
        test_only: &test_only,
    };
    for rule in rules {
        out.findings.extend(rule.check_project(&project_unit));
    }
    for m in &project.manifests {
        let unit = ManifestUnit {
            path: &m.path,
            table: &m.table,
            is_root: m.path == "Cargo.toml",
            project_root: &project.root,
        };
        for rule in rules {
            out.findings.extend(rule.check_manifest(&unit));
        }
    }
    let (kept, suppressed, unused) =
        suppress::apply(&directives, std::mem::take(&mut out.findings));
    out.findings = kept;
    out.suppressed = suppressed;
    out.unused_suppressions = unused;
    out.findings.sort_by_key(|f| f.sort_key());
    out
}

fn parse(path: &Path) -> Result<(syn::File, String), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read file: {e}"))?;
    let ast = syn::parse_file(&text).map_err(|e| {
        let start = e.span().start();
        format!("cannot parse Rust source at line {}: {e}", start.line)
    })?;
    Ok((ast, text))
}

pub fn discover_and_scan(
    root: &Path,
    exclude: &[String],
    rules: &[Box<dyn Rule>],
) -> Result<(Project, ScanOutcome), ProjectError> {
    let project = Project::discover_with(root, exclude)?;
    let outcome = scan_project(&project, rules);
    Ok((project, outcome))
}

/// Scans every Soroban project below `root` and merges the results into one
/// outcome. Each project is analyzed on its own, exactly as `discover_and_scan`
/// would, and its file paths are prefixed with the project's directory so they
/// stay relative to `root`. `exclude` entries are relative to `root`.
pub fn discover_and_scan_all(
    root: &Path,
    exclude: &[String],
    rules: &[Box<dyn Rule>],
) -> Result<(Vec<String>, ScanOutcome), ProjectError> {
    let projects = discover_projects(root, exclude)?;
    let mut merged = ScanOutcome::default();
    for dir in &projects {
        let nested: Vec<String> = exclude
            .iter()
            .filter_map(|x| {
                if dir.is_empty() {
                    Some(x.clone())
                } else {
                    x.strip_prefix(&format!("{dir}/")).map(str::to_string)
                }
            })
            .collect();
        let path = if dir.is_empty() {
            root.to_path_buf()
        } else {
            root.join(dir)
        };
        let (_, mut outcome) = discover_and_scan(&path, &nested, rules)?;
        if !dir.is_empty() {
            let prefix = |f: &mut String| *f = format!("{dir}/{f}");
            outcome
                .findings
                .iter_mut()
                .for_each(|x| prefix(&mut x.file));
            outcome.errors.iter_mut().for_each(|x| prefix(&mut x.file));
            outcome
                .suppressed
                .iter_mut()
                .for_each(|x| prefix(&mut x.file));
            outcome
                .unused_suppressions
                .iter_mut()
                .for_each(|x| prefix(&mut x.file));
        }
        merged.files_scanned += outcome.files_scanned;
        merged.findings.extend(outcome.findings);
        merged.errors.extend(outcome.errors);
        merged.suppressed.extend(outcome.suppressed);
        merged
            .unused_suppressions
            .extend(outcome.unused_suppressions);
    }
    merged.findings.sort_by_key(|f| f.sort_key());
    Ok((projects, merged))
}
