use crate::finding::Finding;
use crate::project::{Project, ProjectError};
use crate::rule::{relative_display, Rule, SourceUnit};
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
}

pub fn scan_project(project: &Project, rules: &[Box<dyn Rule>]) -> ScanOutcome {
    let mut out = ScanOutcome::default();
    for path in &project.sources {
        let rel = relative_display(&project.root, path);
        match parse(path) {
            Ok(ast) => {
                out.files_scanned += 1;
                let unit = SourceUnit {
                    path: &rel,
                    ast: &ast,
                };
                for rule in rules {
                    out.findings.extend(rule.check(&unit));
                }
            }
            Err(message) => out.errors.push(ScanError { file: rel, message }),
        }
    }
    out.findings.sort_by_key(|f| f.sort_key());
    out
}

fn parse(path: &Path) -> Result<syn::File, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("cannot read file: {e}"))?;
    syn::parse_file(&text).map_err(|e| {
        let start = e.span().start();
        format!("cannot parse Rust source at line {}: {e}", start.line)
    })
}

pub fn discover_and_scan(
    root: &Path,
    rules: &[Box<dyn Rule>],
) -> Result<(Project, ScanOutcome), ProjectError> {
    let project = Project::discover(root)?;
    let outcome = scan_project(&project, rules);
    Ok((project, outcome))
}
