pub mod common;
pub mod ss001;
pub mod ss002;
pub mod ss003;
pub mod ss004;
pub mod ss005;
pub mod ss006;

use sorobanshield_core::{Rule, RuleMeta};

/// Rules that are implemented and run during a scan.
pub fn active_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(ss001::AuthorizationPathReview),
        Box::new(ss002::ArithmeticBoundaryReview),
        Box::new(ss003::ExternalCallReview),
        Box::new(ss004::ErrorPathReview),
        Box::new(ss005::StateTransitionTestSuggestions),
        Box::new(ss006::ToolchainDiagnostics),
    ]
}

/// Metadata for every implemented rule, ordered by ID.
pub fn catalog() -> Vec<RuleMeta> {
    let mut all: Vec<RuleMeta> = active_rules().iter().map(|r| r.meta()).collect();
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
    fn catalog_lists_exactly_the_active_rules() {
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
}
