use sorobanshield_core::toml::{Table, Value};
use sorobanshield_core::{Confidence, Finding, ManifestUnit, Rule, RuleMeta, RuleStatus, Severity};

pub const ID: &str = "SS-006";

pub struct ToolchainDiagnostics;

const CARGO_PROFILES: &str =
    "https://doc.rust-lang.org/cargo/reference/profiles.html#overflow-checks";
const CARGO_DEPS: &str = "https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html";
const CARGO_LOCK: &str =
    "https://doc.rust-lang.org/cargo/faq.html#why-have-cargolock-in-version-control";
const TOOLCHAIN_FILE: &str = "https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file";

fn dependency_tables(t: &Table) -> Vec<&Table> {
    let mut v: Vec<&Table> = ["dependencies", "dev-dependencies", "build-dependencies"]
        .iter()
        .filter_map(|k| t.get(*k).and_then(Value::as_table))
        .collect();
    if let Some(w) = t
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .and_then(Value::as_table)
    {
        v.push(w);
    }
    v
}

/// Why a dependency specification is not reproducible, if it is not.
fn unpinned_reason(dep: &Value) -> Option<&'static str> {
    match dep {
        Value::String(s) if s.trim() == "*" => Some("uses the wildcard version `*`"),
        Value::Table(t) => {
            if t.get("version").and_then(Value::as_str).map(str::trim) == Some("*") {
                Some("uses the wildcard version `*`")
            } else if t.contains_key("git") && !t.contains_key("rev") && !t.contains_key("tag") {
                Some("is a git dependency without a `rev` or `tag`")
            } else {
                None
            }
        }
        _ => None,
    }
}

fn builds_cdylib(t: &Table) -> bool {
    t.get("lib")
        .and_then(|l| l.get("crate-type"))
        .and_then(Value::as_array)
        .is_some_and(|a| a.iter().any(|v| v.as_str() == Some("cdylib")))
}

fn overflow_checks_enabled(t: &Table) -> bool {
    t.get("profile")
        .and_then(|p| p.get("release"))
        .and_then(|r| r.get("overflow-checks"))
        .and_then(Value::as_bool)
        == Some(true)
}

fn diag(
    meta: &RuleMeta,
    unit: &ManifestUnit<'_>,
    confidence: Confidence,
    message: String,
    remediation: &str,
    reference: &str,
    limitation: &str,
) -> Finding {
    Finding {
        rule_id: ID.into(),
        title: meta.name.clone(),
        severity: Severity::Info,
        confidence,
        message,
        file: unit.path.to_string(),
        start_line: None,
        end_line: None,
        remediation: remediation.into(),
        references: vec![reference.into()],
        limitations: vec![limitation.into()],
    }
}

impl Rule for ToolchainDiagnostics {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: ID.into(),
            name: "Dependency and toolchain diagnostics".into(),
            status: RuleStatus::Implemented,
            severity: Some(Severity::Info),
            confidence: Some(Confidence::High),
            concern: "Project metadata that makes builds hard to reproduce or that weakens arithmetic safety checks: unpinned soroban-sdk, a missing cdylib crate type, release builds without overflow checks, and a missing Cargo.lock or Rust toolchain file.".into(),
            detection: "Reads each Cargo.toml (parsed with the toml crate) and checks for five conditions: (1) soroban-sdk declared as `*` or as a git dependency without rev/tag in any dependency table; (2) a [package] manifest that depends on soroban-sdk but whose [lib] crate-type lacks cdylib; (3) the root manifest not setting [profile.release] overflow-checks = true; (4) no Cargo.lock beside the root manifest; (5) no rust-toolchain.toml or rust-toolchain beside the root manifest. It makes no claim about known vulnerabilities in any dependency version, and does not consult any advisory database.".into(),
            severity_rationale: "Informational: these are reproducibility and hardening hints, not vulnerabilities. Confidence is high for the file-presence and exact-value checks because they are mechanical; the overflow-checks finding is medium because the setting can also be supplied by .cargo/config.toml or environment variables that are not read.".into(),
            remediation: "Pin soroban-sdk to an exact or caret version (or a git rev/tag), build contracts as `crate-type = [\"cdylib\"]`, set `overflow-checks = true` under `[profile.release]`, commit Cargo.lock, and pin the toolchain with a rust-toolchain.toml.".into(),
            false_positives: vec![
                "overflow-checks supplied through .cargo/config.toml or CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS, which are not read.".into(),
                "Helper or library crates that depend on soroban-sdk but are intentionally not compiled to WASM contracts (no cdylib needed).".into(),
                "Toolchain pinned by CI configuration or a parent directory's rust-toolchain file, and Cargo.lock deliberately excluded.".into(),
            ],
            false_negatives: vec![
                "Known-vulnerable or outdated dependency versions: no advisory data is consulted.".into(),
                "Inconsistent soroban-sdk versions across workspace members, or a toolchain that does not match the SDK's requirements.".into(),
                "Settings inherited through [workspace.package] or profile `inherits` chains are not resolved.".into(),
                "Profile settings in non-root manifests are ignored, which matches Cargo's behavior but can hide a mistaken member-level profile.".into(),
            ],
            references: vec![CARGO_PROFILES.into(), CARGO_DEPS.into(), CARGO_LOCK.into(), TOOLCHAIN_FILE.into()],
            fixtures: vec![
                "tests/fixtures/positive/poor_metadata".into(),
                "tests/fixtures/negative/good_metadata".into(),
            ],
        }
    }

    fn check_manifest(&self, unit: &ManifestUnit<'_>) -> Vec<Finding> {
        let meta = self.meta();
        let t = unit.table;
        let mut out = Vec::new();

        for deps in dependency_tables(t) {
            if let Some(reason) = deps.get("soroban-sdk").and_then(unpinned_reason) {
                out.push(diag(
                    &meta,
                    unit,
                    Confidence::High,
                    format!("The soroban-sdk dependency {reason}, so builds are not reproducible."),
                    "Pin soroban-sdk to a specific version requirement, or a git rev/tag.",
                    CARGO_DEPS,
                    "Does not check whether the pinned version is current or free of known issues.",
                ));
            }
        }

        let depends_on_sdk = t
            .get("dependencies")
            .and_then(Value::as_table)
            .is_some_and(|d| d.contains_key("soroban-sdk"));
        if t.contains_key("package") && depends_on_sdk && !builds_cdylib(t) {
            out.push(diag(
                &meta,
                unit,
                Confidence::High,
                "This package depends on soroban-sdk but its [lib] crate-type does not include \"cdylib\", which Soroban contract builds need to produce WASM.".into(),
                "Add `[lib]` with `crate-type = [\"cdylib\"]` if this crate is a contract.",
                CARGO_DEPS,
                "Library crates that are not contracts legitimately omit cdylib.",
            ));
        }

        if unit.is_root {
            if !overflow_checks_enabled(t) {
                out.push(diag(
                    &meta,
                    unit,
                    Confidence::Medium,
                    "[profile.release] does not set overflow-checks = true, so integer overflow in release builds may wrap silently instead of trapping.".into(),
                    "Set `overflow-checks = true` under `[profile.release]` unless wrapping arithmetic is intended everywhere.",
                    CARGO_PROFILES,
                    "The setting may be provided by .cargo/config.toml or environment variables, which this rule does not read.",
                ));
            }
            if !unit.project_root.join("Cargo.lock").is_file() {
                out.push(diag(
                    &meta,
                    unit,
                    Confidence::High,
                    "No Cargo.lock beside the root manifest, so dependency versions can drift between builds.".into(),
                    "Run `cargo generate-lockfile` and commit Cargo.lock.",
                    CARGO_LOCK,
                    "Only the presence of the file is checked, not whether it is committed or current.",
                ));
            }
            let root = unit.project_root;
            if !root.join("rust-toolchain.toml").is_file() && !root.join("rust-toolchain").is_file()
            {
                out.push(diag(
                    &meta,
                    unit,
                    Confidence::High,
                    "No rust-toolchain.toml (or rust-toolchain) beside the root manifest, so the compiler version is not pinned for this project.".into(),
                    "Add a rust-toolchain.toml that pins the channel and the wasm32 target your contract build needs.",
                    TOOLCHAIN_FILE,
                    "A toolchain pinned by CI or a parent directory is not detected.",
                ));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn check(manifest: &str, root: &Path, is_root: bool) -> Vec<Finding> {
        let table: Table = manifest.parse().unwrap();
        ToolchainDiagnostics.check_manifest(&ManifestUnit {
            path: "Cargo.toml",
            table: &table,
            is_root,
            project_root: root,
        })
    }

    const GOOD: &str = "[package]\nname=\"c\"\nversion=\"0.1.0\"\n[lib]\ncrate-type=[\"cdylib\"]\n[dependencies]\nsoroban-sdk=\"22\"\n[profile.release]\noverflow-checks=true\n";

    fn complete_root() -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("Cargo.lock"), "").unwrap();
        fs::write(d.path().join("rust-toolchain.toml"), "").unwrap();
        d
    }

    #[test]
    fn complete_project_has_no_findings() {
        let d = complete_root();
        assert!(check(GOOD, d.path(), true).is_empty());
    }

    #[test]
    fn flags_wildcard_and_unpinned_git_sdk() {
        let d = complete_root();
        let wild = GOOD.replace("\"22\"", "\"*\"");
        assert_eq!(check(&wild, d.path(), true).len(), 1);
        let tbl = GOOD.replace(
            "soroban-sdk=\"22\"",
            "soroban-sdk={ git = \"https://example.com/x\" }",
        );
        assert_eq!(check(&tbl, d.path(), true).len(), 1);
        let pinned = GOOD.replace(
            "soroban-sdk=\"22\"",
            "soroban-sdk={ git = \"https://example.com/x\", rev = \"abc\" }",
        );
        assert!(check(&pinned, d.path(), true).is_empty());
    }

    #[test]
    fn flags_wildcard_in_workspace_dependencies() {
        let d = complete_root();
        let m = format!("{GOOD}[workspace.dependencies]\nsoroban-sdk = \"*\"\n");
        assert_eq!(check(&m, d.path(), true).len(), 1);
    }

    #[test]
    fn flags_missing_cdylib() {
        let d = complete_root();
        let m = GOOD.replace("[lib]\ncrate-type=[\"cdylib\"]\n", "");
        let f = check(&m, d.path(), true);
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("cdylib"));
    }

    #[test]
    fn dev_dependency_only_does_not_require_cdylib() {
        let d = complete_root();
        let m = "[package]\nname=\"c\"\nversion=\"0.1.0\"\n[dev-dependencies]\nsoroban-sdk=\"22\"\n[profile.release]\noverflow-checks=true\n";
        assert!(check(m, d.path(), true).is_empty());
    }

    #[test]
    fn overflow_checks_must_be_true_in_root_release_profile() {
        let d = complete_root();
        let off = GOOD.replace("overflow-checks=true", "overflow-checks=false");
        let f = check(&off, d.path(), true);
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].confidence, Confidence::Medium);
        let missing = GOOD.replace("[profile.release]\noverflow-checks=true\n", "");
        assert_eq!(check(&missing, d.path(), true).len(), 1);
    }

    #[test]
    fn non_root_manifests_skip_profile_and_file_checks() {
        let d = tempfile::tempdir().unwrap();
        let member = GOOD.replace("[profile.release]\noverflow-checks=true\n", "");
        assert!(check(&member, d.path(), false).is_empty());
    }

    #[test]
    fn flags_missing_lockfile_and_toolchain() {
        let d = tempfile::tempdir().unwrap();
        let f = check(GOOD, d.path(), true);
        assert_eq!(f.len(), 2);
        assert!(f.iter().any(|x| x.message.contains("Cargo.lock")));
        assert!(f.iter().any(|x| x.message.contains("rust-toolchain")));
        fs::write(d.path().join("rust-toolchain"), "").unwrap();
        assert_eq!(check(GOOD, d.path(), true).len(), 1);
    }

    #[test]
    fn findings_are_informational_without_line_numbers() {
        let d = tempfile::tempdir().unwrap();
        for f in check(
            "[package]\nname=\"c\"\nversion=\"0.1.0\"\n[dependencies]\nsoroban-sdk=\"*\"\n",
            d.path(),
            true,
        ) {
            assert_eq!(f.severity, Severity::Info);
            assert_eq!(f.start_line, None);
            assert_eq!(f.rule_id, "SS-006");
        }
    }
}
