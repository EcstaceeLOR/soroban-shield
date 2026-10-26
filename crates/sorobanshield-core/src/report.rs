use crate::engine::{ScanError, ScanOutcome};
use crate::finding::{Finding, Severity};
use crate::rule::RuleMeta;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::Write;

pub const SCHEMA_VERSION: u32 = 1;
pub const DISCLAIMER: &str = "Static analysis cannot prove a contract is secure. A scan with no findings only means the implemented checks reported nothing.";

#[derive(Debug, Serialize)]
pub struct Tool {
    pub name: &'static str,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct ScanInfo {
    pub root: String,
    pub files_scanned: usize,
    pub rules_run: Vec<String>,
    /// Path of the configuration file that was applied, if any.
    pub config_file: Option<String>,
    /// Path prefixes excluded by configuration.
    pub exclude: Vec<String>,
    /// Rules disabled by configuration.
    pub disabled_rules: Vec<String>,
    /// Baseline file that was applied, if any.
    pub baseline_file: Option<String>,
    /// Baseline entries that matched no current finding.
    pub baseline_unmatched: usize,
    pub errors: Vec<ScanError>,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    /// Findings reported (after baseline suppression).
    pub total: usize,
    /// Findings hidden because they are listed in the baseline.
    pub suppressed_by_baseline: usize,
    pub by_severity: BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub tool: Tool,
    pub scan: ScanInfo,
    pub summary: Summary,
    pub findings: Vec<Finding>,
    pub disclaimer: &'static str,
}

impl Report {
    pub fn new(version: &str, root: &str, rules_run: Vec<String>, outcome: ScanOutcome) -> Report {
        let mut by_severity: BTreeMap<String, usize> =
            Severity::ALL.iter().map(|s| (s.to_string(), 0)).collect();
        for f in &outcome.findings {
            *by_severity.entry(f.severity.to_string()).or_default() += 1;
        }
        Report {
            schema_version: SCHEMA_VERSION,
            tool: Tool {
                name: "sorobanshield",
                version: version.to_string(),
            },
            scan: ScanInfo {
                root: root.to_string(),
                files_scanned: outcome.files_scanned,
                rules_run,
                config_file: None,
                exclude: Vec::new(),
                disabled_rules: Vec::new(),
                baseline_file: None,
                baseline_unmatched: 0,
                errors: outcome.errors,
            },
            summary: Summary {
                total: outcome.findings.len(),
                suppressed_by_baseline: 0,
                by_severity,
            },
            findings: outcome.findings,
            disclaimer: DISCLAIMER,
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.scan.errors.is_empty()
    }

    pub fn exceeds(&self, threshold: Severity) -> bool {
        self.findings.iter().any(|f| f.severity >= threshold)
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("report serialises")
    }

    /// SARIF 2.1.0 log. `metas` describes the rules that ran.
    pub fn to_sarif(&self, metas: &[RuleMeta]) -> String {
        use serde_json::{json, Value};

        fn level(s: Option<Severity>) -> &'static str {
            match s {
                Some(Severity::Critical | Severity::High) => "error",
                Some(Severity::Medium) => "warning",
                _ => "note",
            }
        }
        fn location(file: &str, region: Option<Value>) -> Value {
            let mut physical = json!({
                "artifactLocation": { "uri": file, "uriBaseId": "%SRCROOT%" }
            });
            if let Some(r) = region {
                physical["region"] = r;
            }
            json!({ "physicalLocation": physical })
        }

        let rules: Vec<Value> = metas
            .iter()
            .map(|m| {
                json!({
                    "id": m.id,
                    "name": m.name.split_whitespace().collect::<String>(),
                    "shortDescription": { "text": m.name },
                    "fullDescription": { "text": m.concern },
                    "help": { "text": m.remediation },
                    "defaultConfiguration": { "level": level(m.severity) },
                    "properties": {
                        "severity": m.severity.map(|s| s.to_string()),
                        "confidence": m.confidence.map(|c| c.to_string()),
                    }
                })
            })
            .collect();

        let results: Vec<Value> = self
            .findings
            .iter()
            .map(|f| {
                let region = f
                    .start_line
                    .map(|s| json!({ "startLine": s, "endLine": f.end_line.unwrap_or(s) }));
                json!({
                    "ruleId": f.rule_id,
                    "ruleIndex": metas.iter().position(|m| m.id == f.rule_id),
                    "level": level(Some(f.severity)),
                    "message": { "text": format!("{} Next step: {}", f.message, f.remediation) },
                    "locations": [location(&f.file, region)],
                    "properties": {
                        "severity": f.severity.to_string(),
                        "confidence": f.confidence.to_string(),
                        "limitations": f.limitations,
                    }
                })
            })
            .collect();

        let notifications: Vec<Value> = self
            .scan
            .errors
            .iter()
            .map(|e| {
                json!({
                    "level": "error",
                    "message": { "text": e.message },
                    "locations": [location(&e.file, None)],
                })
            })
            .collect();

        let log = json!({
            "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": { "driver": {
                    "name": "SorobanShield",
                    "version": self.tool.version,
                    "informationUri": "https://github.com/sorobanshield/soroban-shield",
                    "rules": rules,
                }},
                "results": results,
                "invocations": [{
                    "executionSuccessful": self.scan.errors.is_empty(),
                    "toolExecutionNotifications": notifications,
                }],
                "properties": { "disclaimer": DISCLAIMER },
            }]
        });
        serde_json::to_string_pretty(&log).expect("sarif serialises")
    }

    pub fn to_terminal(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(
            s,
            "SorobanShield {} - scanned {} file(s) with {} rule(s)",
            self.tool.version,
            self.scan.files_scanned,
            self.scan.rules_run.len()
        );
        if let Some(c) = &self.scan.config_file {
            let _ = writeln!(
                s,
                "config: {c} ({} excluded path(s), {} rule(s) disabled)",
                self.scan.exclude.len(),
                self.scan.disabled_rules.len()
            );
        }
        if let Some(b) = &self.scan.baseline_file {
            let _ = writeln!(
                s,
                "baseline: {b} ({} finding(s) suppressed, {} stale entr(ies))",
                self.summary.suppressed_by_baseline, self.scan.baseline_unmatched
            );
        }
        for e in &self.scan.errors {
            let _ = writeln!(s, "error: {}: {}", e.file, e.message);
        }
        if self.findings.is_empty() {
            if self.has_errors() {
                let _ = writeln!(s, "\nScan incomplete: some files could not be analyzed. This is NOT a clean result.");
            } else {
                let _ = writeln!(s, "\nNo findings reported by the implemented checks.");
            }
        }
        for (i, f) in self.findings.iter().enumerate() {
            let loc = match (f.start_line, f.end_line) {
                (Some(a), Some(b)) if a != b => format!("{}:{}-{}", f.file, a, b),
                (Some(a), _) => format!("{}:{}", f.file, a),
                _ => f.file.clone(),
            };
            let _ = writeln!(
                s,
                "\n[{}] {} {} (severity: {}, confidence: {})",
                i + 1,
                f.rule_id,
                f.title,
                f.severity,
                f.confidence
            );
            let _ = writeln!(s, "    at {loc}");
            let _ = writeln!(s, "    {}", f.message);
            let _ = writeln!(s, "    next step: {}", f.remediation);
            for l in &f.limitations {
                let _ = writeln!(s, "    limitation: {l}");
            }
        }
        let _ = writeln!(s, "\n{} finding(s). {}", self.summary.total, DISCLAIMER);
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::Confidence;

    fn finding(sev: Severity, file: &str) -> Finding {
        Finding {
            rule_id: "SS-001".into(),
            title: "t".into(),
            severity: sev,
            confidence: Confidence::Low,
            message: "m \"quoted\"".into(),
            file: file.into(),
            start_line: Some(3),
            end_line: Some(5),
            remediation: "r".into(),
            references: vec![],
            limitations: vec!["l".into()],
        }
    }

    fn report(findings: Vec<Finding>, errors: Vec<ScanError>) -> Report {
        Report::new(
            "0.0.0",
            ".",
            vec!["SS-001".into()],
            ScanOutcome {
                files_scanned: 1,
                findings,
                errors,
            },
        )
    }

    #[test]
    fn json_is_valid_deterministic_and_complete() {
        let r = report(vec![finding(Severity::Medium, "a.rs")], vec![]);
        let a = r.to_json();
        assert_eq!(a, r.to_json());
        let v: serde_json::Value = serde_json::from_str(&a).unwrap();
        assert_eq!(v["schema_version"], 1);
        assert_eq!(v["summary"]["total"], 1);
        assert_eq!(v["summary"]["by_severity"]["medium"], 1);
        assert_eq!(v["summary"]["by_severity"]["high"], 0);
        assert_eq!(v["findings"][0]["message"], "m \"quoted\"");
        assert_eq!(v["findings"][0]["start_line"], 3);
    }

    #[test]
    fn threshold_logic() {
        let r = report(vec![finding(Severity::Low, "a.rs")], vec![]);
        assert!(r.exceeds(Severity::Low));
        assert!(!r.exceeds(Severity::Medium));
    }

    #[test]
    fn terminal_clean_vs_incomplete() {
        assert!(report(vec![], vec![])
            .to_terminal()
            .contains("No findings reported"));
        let t = report(
            vec![],
            vec![ScanError {
                file: "x.rs".into(),
                message: "boom".into(),
            }],
        )
        .to_terminal();
        assert!(t.contains("NOT a clean result"));
    }

    #[test]
    fn terminal_shows_location_and_fields() {
        let t = report(vec![finding(Severity::Medium, "src/lib.rs")], vec![]).to_terminal();
        assert!(t.contains("src/lib.rs:3-5"));
        assert!(t.contains("severity: medium, confidence: low"));
        assert!(t.contains("limitation: l"));
    }
}
