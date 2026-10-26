use crate::finding::{Confidence, Finding, Severity};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleStatus {
    Planned,
    Experimental,
    Implemented,
    Deprecated,
}

/// Static description of a rule. Serialised to JSON so the website catalog
/// is generated from the same data the CLI uses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMeta {
    pub id: String,
    pub name: String,
    pub status: RuleStatus,
    pub severity: Option<Severity>,
    pub confidence: Option<Confidence>,
    pub concern: String,
    pub detection: String,
    pub severity_rationale: String,
    pub remediation: String,
    pub false_positives: Vec<String>,
    pub false_negatives: Vec<String>,
    pub references: Vec<String>,
    pub fixtures: Vec<String>,
}

/// A parsed Rust source file handed to rules.
pub struct SourceUnit<'a> {
    /// Path relative to the scan root, `/`-separated.
    pub path: &'a str,
    pub ast: &'a syn::File,
}

pub trait Rule: Sync {
    fn meta(&self) -> RuleMeta;
    fn check(&self, unit: &SourceUnit<'_>) -> Vec<Finding>;
}

pub fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}
