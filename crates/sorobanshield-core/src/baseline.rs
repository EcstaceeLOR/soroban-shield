use crate::finding::Finding;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;
use thiserror::Error;

pub const BASELINE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum BaselineError {
    #[error("cannot read baseline file {path}: {reason}")]
    Read { path: String, reason: String },
    #[error("invalid baseline file {path}: {reason}")]
    Parse { path: String, reason: String },
    #[error("cannot write baseline file {path}: {reason}")]
    Write { path: String, reason: String },
}

/// Findings accepted as known. Entries identify a finding by rule, file and
/// message, never by line number, so they survive unrelated edits above them.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Baseline {
    pub schema_version: u32,
    pub fingerprints: BTreeSet<String>,
}

pub struct Applied {
    pub kept: Vec<Finding>,
    pub suppressed: usize,
    /// Baseline entries that matched no current finding (the issue was fixed or changed).
    pub unmatched: usize,
}

pub fn fingerprint(f: &Finding) -> String {
    format!("{}|{}|{}", f.rule_id, f.file, f.message)
}

impl Baseline {
    pub fn from_findings(findings: &[Finding]) -> Baseline {
        Baseline {
            schema_version: BASELINE_SCHEMA_VERSION,
            fingerprints: findings.iter().map(fingerprint).collect(),
        }
    }

    pub fn load(path: &Path) -> Result<Baseline, BaselineError> {
        let shown = path.display().to_string();
        let text = std::fs::read_to_string(path).map_err(|e| BaselineError::Read {
            path: shown.clone(),
            reason: e.to_string(),
        })?;
        let b: Baseline = serde_json::from_str(&text).map_err(|e| BaselineError::Parse {
            path: shown.clone(),
            reason: e.to_string(),
        })?;
        if b.schema_version != BASELINE_SCHEMA_VERSION {
            return Err(BaselineError::Parse {
                path: shown,
                reason: format!(
                    "unsupported schema_version {} (expected {BASELINE_SCHEMA_VERSION})",
                    b.schema_version
                ),
            });
        }
        Ok(b)
    }

    pub fn save(&self, path: &Path) -> Result<(), BaselineError> {
        let text = serde_json::to_string_pretty(self).expect("baseline serialises");
        std::fs::write(path, text + "\n").map_err(|e| BaselineError::Write {
            path: path.display().to_string(),
            reason: e.to_string(),
        })
    }

    pub fn apply(&self, findings: Vec<Finding>) -> Applied {
        let mut matched = BTreeSet::new();
        let mut kept = Vec::new();
        let mut suppressed = 0;
        for f in findings {
            let fp = fingerprint(&f);
            if self.fingerprints.contains(&fp) {
                suppressed += 1;
                matched.insert(fp);
            } else {
                kept.push(f);
            }
        }
        Applied {
            kept,
            suppressed,
            unmatched: self.fingerprints.len() - matched.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::{Confidence, Severity};

    fn f(rule: &str, file: &str, msg: &str, line: usize) -> Finding {
        Finding {
            rule_id: rule.into(),
            title: "t".into(),
            severity: Severity::Medium,
            confidence: Confidence::Low,
            message: msg.into(),
            file: file.into(),
            start_line: Some(line),
            end_line: Some(line),
            remediation: String::new(),
            references: vec![],
            limitations: vec![],
        }
    }

    #[test]
    fn fingerprint_ignores_line_numbers() {
        assert_eq!(
            fingerprint(&f("SS-001", "a.rs", "m", 3)),
            fingerprint(&f("SS-001", "a.rs", "m", 90))
        );
        assert_ne!(
            fingerprint(&f("SS-001", "a.rs", "m", 3)),
            fingerprint(&f("SS-002", "a.rs", "m", 3))
        );
        assert_ne!(
            fingerprint(&f("SS-001", "a.rs", "m", 3)),
            fingerprint(&f("SS-001", "b.rs", "m", 3))
        );
    }

    #[test]
    fn apply_suppresses_known_keeps_new_and_counts_stale() {
        let base = Baseline::from_findings(&[
            f("SS-001", "a.rs", "old", 1),
            f("SS-001", "a.rs", "fixed", 2),
        ]);
        let applied = base.apply(vec![
            f("SS-001", "a.rs", "old", 40),
            f("SS-001", "a.rs", "new", 5),
        ]);
        assert_eq!(applied.suppressed, 1);
        assert_eq!(applied.kept.len(), 1);
        assert_eq!(applied.kept[0].message, "new");
        assert_eq!(applied.unmatched, 1);
    }

    #[test]
    fn round_trips_and_is_sorted_and_deduplicated() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("b.json");
        let base = Baseline::from_findings(&[
            f("SS-002", "z.rs", "m", 1),
            f("SS-001", "a.rs", "m", 1),
            f("SS-001", "a.rs", "m", 2),
        ]);
        assert_eq!(base.fingerprints.len(), 2);
        base.save(&p).unwrap();
        let text = std::fs::read_to_string(&p).unwrap();
        assert!(text.find("SS-001").unwrap() < text.find("SS-002").unwrap());
        assert_eq!(Baseline::load(&p).unwrap(), base);
    }

    #[test]
    fn rejects_bad_files() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("b.json");
        assert!(matches!(
            Baseline::load(&p),
            Err(BaselineError::Read { .. })
        ));
        std::fs::write(&p, "not json").unwrap();
        assert!(matches!(
            Baseline::load(&p),
            Err(BaselineError::Parse { .. })
        ));
        std::fs::write(&p, r#"{"schema_version":1,"fingerprints":[],"extra":1}"#).unwrap();
        assert!(Baseline::load(&p).is_err());
        std::fs::write(&p, r#"{"schema_version":9,"fingerprints":[]}"#).unwrap();
        assert!(Baseline::load(&p).is_err());
    }
}
