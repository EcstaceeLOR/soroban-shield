pub mod planned;
pub mod ss001;

use sorobanshield_core::{Rule, RuleMeta};

/// Rules that are implemented and run during a scan.
pub fn active_rules() -> Vec<Box<dyn Rule>> {
    vec![Box::new(ss001::AuthorizationPathReview)]
}

/// Metadata for every known rule: implemented ones first, then planned ones.
/// Planned rules are catalog entries only and are never executed.
pub fn catalog() -> Vec<RuleMeta> {
    let mut all: Vec<RuleMeta> = active_rules().iter().map(|r| r.meta()).collect();
    all.extend(planned::planned_rules());
    all.sort_by(|a, b| a.id.cmp(&b.id));
    all
}

pub fn find_rule(id: &str) -> Option<RuleMeta> {
    catalog()
        .into_iter()
        .find(|m| m.id.eq_ignore_ascii_case(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sorobanshield_core::RuleStatus;

    #[test]
    fn only_implemented_rules_are_active() {
        let active: Vec<String> = active_rules().iter().map(|r| r.meta().id).collect();
        let implemented: Vec<String> = catalog()
            .into_iter()
            .filter(|m| m.status == RuleStatus::Implemented)
            .map(|m| m.id)
            .collect();
        assert_eq!(active, implemented);
    }

    #[test]
    fn rule_ids_are_unique_and_well_formed() {
        let ids: Vec<String> = catalog().into_iter().map(|m| m.id).collect();
        let mut dedup = ids.clone();
        dedup.dedup();
        assert_eq!(ids, dedup);
        assert!(ids.iter().all(|i| i.starts_with("SS-") && i.len() == 6));
    }

    #[test]
    fn lookup_is_case_insensitive() {
        assert!(find_rule("ss-001").is_some());
        assert!(find_rule("SS-999").is_none());
    }

    #[test]
    fn planned_rules_make_no_severity_claim() {
        for m in catalog()
            .into_iter()
            .filter(|m| m.status == RuleStatus::Planned)
        {
            assert!(
                m.severity.is_none() && m.confidence.is_none(),
                "{} must not claim severity",
                m.id
            );
        }
    }
}
