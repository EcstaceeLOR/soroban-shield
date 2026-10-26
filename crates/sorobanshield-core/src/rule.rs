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

/// A parsed `Cargo.toml` handed to rules.
pub struct ManifestUnit<'a> {
    /// Path relative to the scan root, `/`-separated.
    pub path: &'a str,
    pub table: &'a toml::Table,
    /// True for the manifest at the scan root.
    pub is_root: bool,
    /// The scan root, for checking whether sibling project files exist.
    pub project_root: &'a Path,
}

/// All parsed source files of the project, for rules that need cross-file context.
pub struct ProjectUnit<'a> {
    pub files: &'a [SourceUnit<'a>],
}

pub trait Rule: Sync {
    fn meta(&self) -> RuleMeta;
    /// Inspect a parsed Rust source file. Defaults to no findings.
    fn check(&self, _unit: &SourceUnit<'_>) -> Vec<Finding> {
        Vec::new()
    }
    /// Inspect every parsed source file together. Defaults to no findings.
    fn check_project(&self, _unit: &ProjectUnit<'_>) -> Vec<Finding> {
        Vec::new()
    }
    /// Inspect a parsed Cargo.toml. Defaults to no findings.
    fn check_manifest(&self, _unit: &ManifestUnit<'_>) -> Vec<Finding> {
        Vec::new()
    }
}

pub fn relative_display(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}
