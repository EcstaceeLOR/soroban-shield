use crate::finding::Finding;
use crate::project::{Project, ProjectError};
use crate::rule::{relative_display, ManifestUnit, ProjectUnit, Rule, SourceUnit};
use crate::suppress::{self, Suppression, UnusedSuppression};
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
    for unit in &units {
        for rule in rules {
            out.findings.extend(rule.check(unit));
        }
    }
    let project_unit = ProjectUnit { files: &units };
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
