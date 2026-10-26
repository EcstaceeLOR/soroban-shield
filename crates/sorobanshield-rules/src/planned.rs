use sorobanshield_core::{RuleMeta, RuleStatus};

fn planned(id: &str, name: &str, concern: &str, approach: &str, reference: &str) -> RuleMeta {
    RuleMeta {
        id: id.into(),
        name: name.into(),
        status: RuleStatus::Planned,
        severity: None,
        confidence: None,
        concern: concern.into(),
        detection: approach.into(),
        severity_rationale:
            "Not assigned: this rule is not implemented and the scanner does not run it.".into(),
        remediation: "Not applicable until the rule is implemented.".into(),
        false_positives: vec![],
        false_negatives: vec![],
        references: vec![reference.into()],
        fixtures: vec![],
    }
}

pub fn planned_rules() -> Vec<RuleMeta> {
    let sdk = "https://developers.stellar.org/docs/build/smart-contracts";
    vec![
        planned("SS-003", "External contract call review", "Cross-contract and token calls have authorization and state-ordering implications.", "Planned: locate selected external calls and prompt review where result handling is unclear.", sdk),
        planned("SS-004", "Error-path review", "Inconsistent error handling can leave unexpected state changes in place.", "Planned: a narrowly defined set of suspicious error-handling patterns.", sdk),
        planned("SS-005", "State-transition test suggestions", "Individually correct functions can combine into incorrect state transitions.", "Planned: test suggestions for selected stateful patterns, clearly marked as suggestions.", sdk),
        planned("SS-006", "Dependency and toolchain diagnostics", "An inconsistent toolchain makes builds and checks hard to reproduce.", "Planned: report missing or inconsistent project metadata when reliably detectable.", sdk),
    ]
}
