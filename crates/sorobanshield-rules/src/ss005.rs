use crate::common::{contract_entrypoints, has_attr, is_cfg_test, storage_write_spans};
use proc_macro2::{TokenStream, TokenTree};
use sorobanshield_core::{
    Confidence, Finding, ProjectUnit, Rule, RuleMeta, RuleStatus, Severity, SourceUnit,
};
use std::collections::HashSet;
use syn::visit::Visit;
use syn::{ImplItem, Item};

pub const ID: &str = "SS-005";

pub struct StateTransitionTestSuggestions;

/// Collects every identifier in an AST, including identifiers inside macro arguments
/// (for example `assert_eq!(client.set_value(&1), ..)`), which syn leaves unparsed.
#[derive(Default)]
struct Idents(HashSet<String>);

impl Idents {
    fn tokens(&mut self, ts: TokenStream) {
        for tt in ts {
            match tt {
                TokenTree::Ident(i) => {
                    self.0.insert(i.to_string());
                }
                TokenTree::Group(g) => self.tokens(g.stream()),
                _ => {}
            }
        }
    }
}

impl<'ast> Visit<'ast> for Idents {
    fn visit_ident(&mut self, i: &'ast proc_macro2::Ident) {
        self.0.insert(i.to_string());
    }
    fn visit_macro(&mut self, m: &'ast syn::Macro) {
        self.tokens(m.tokens.clone());
        syn::visit::visit_macro(self, m);
    }
}

fn has_test_fn(items: &[Item]) -> bool {
    items.iter().any(|item| match item {
        Item::Fn(f) => has_attr(&f.attrs, "test"),
        Item::Mod(m) => m.content.as_ref().is_some_and(|(_, i)| has_test_fn(i)),
        Item::Impl(i) => i
            .items
            .iter()
            .any(|x| matches!(x, ImplItem::Fn(f) if has_attr(&f.attrs, "test"))),
        _ => false,
    })
}

/// Identifiers that appear in test code of one file.
fn test_idents(unit: &SourceUnit<'_>, defines_contract: bool) -> Option<HashSet<String>> {
    let mut ids = Idents::default();
    let mut any = false;
    if !defines_contract && has_test_fn(&unit.ast.items) {
        ids.visit_file(unit.ast);
        any = true;
    } else {
        for item in &unit.ast.items {
            let attrs = match item {
                Item::Mod(m) => Some(&m.attrs),
                Item::Fn(f) => Some(&f.attrs),
                _ => None,
            };
            if let Some(a) = attrs {
                if is_cfg_test(a) || has_attr(a, "test") {
                    ids.visit_item(item);
                    any = true;
                }
            }
        }
    }
    any.then_some(ids.0)
}

impl Rule for StateTransitionTestSuggestions {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: ID.into(),
            name: "State-transition test suggestions".into(),
            status: RuleStatus::Implemented,
            severity: Some(Severity::Info),
            confidence: Some(Confidence::Low),
            concern: "A public contract function that writes storage is a state transition. If no test in the project appears to call it, regressions in its authorization, arithmetic and sequencing with other transitions go unnoticed.".into(),
            detection: "Project-wide. Collects public #[contractimpl] functions that write storage (.set / .remove on a .storage() chain), then collects identifiers from test code: #[cfg(test)] modules and #[test] functions, plus whole files that contain #[test] functions but define no contract. It reports each state-changing entrypoint whose name appears in no test code; if the project has no test code at all, it reports one finding listing the entrypoints. Matching is by name, including inside macro arguments such as assert_eq!. The __constructor function is not reported because tests start a contract through registration arguments and never name it. Files that a parent declares with #[cfg(test)] mod name; count as test code and contribute no entrypoints.".into(),
            severity_rationale: "Informational: missing tests are a prompt to write tests, not a defect in the contract. Low confidence because matching is by name only, tests in other crates or generated clients under different names are not seen, and a name match does not mean the transition is meaningfully tested.".into(),
            remediation: "Add tests that call the entrypoint as an authorized and an unauthorized caller, exercise boundary values, and cover sequences of transitions that depend on one another (for example initialize, update, then update again).".into(),
            false_positives: vec![
                "Tests live in another crate or repository, or call the contract through a differently named wrapper.".into(),
                "Entrypoints exercised only through a generic helper that receives the function name as a string.".into(),
            ],
            false_negatives: vec![
                "A name appears in test code without the transition being meaningfully tested, for example a call whose result is never checked.".into(),
                "Name collisions: another identifier with the same name in test code counts as coverage.".into(),
                "Sequences of transitions are never inspected; only individual entrypoints are considered.".into(),
            ],
            references: vec![
                "https://developers.stellar.org/docs/build/smart-contracts/getting-started/storing-data".into(),
                "https://doc.rust-lang.org/book/ch11-01-writing-tests.html".into(),
            ],
            fixtures: vec![
                "tests/fixtures/positive/untested_entrypoints".into(),
                "tests/fixtures/negative/tested_entrypoints".into(),
                "tests/fixtures/edge/partially_tested".into(),
            ],
        }
    }

    fn check_project(&self, project: &ProjectUnit<'_>) -> Vec<Finding> {
        let meta = self.meta();
        let mut entrypoints: Vec<(&str, String, usize)> = Vec::new();
        let mut tested: HashSet<String> = HashSet::new();
        let mut any_test_code = false;

        for unit in project.files {
            let is_test_file = project.test_only.contains(unit.path);
            let fns = if is_test_file {
                Vec::new()
            } else {
                contract_entrypoints(&unit.ast.items)
            };
            for f in &fns {
                // Tests start a contract through registration arguments and never
                // name `__constructor`, so a name match would always miss it.
                if f.sig.ident == "__constructor" {
                    continue;
                }
                if !storage_write_spans(&f.block).is_empty() {
                    entrypoints.push((
                        unit.path,
                        f.sig.ident.to_string(),
                        f.sig.ident.span().start().line,
                    ));
                }
            }
            if let Some(ids) = test_idents(unit, !fns.is_empty()) {
                any_test_code = true;
                tested.extend(ids);
            }
        }
        if entrypoints.is_empty() {
            return Vec::new();
        }

        let finding = |file: &str, line: usize, message: String| Finding {
            rule_id: ID.into(),
            title: meta.name.clone(),
            severity: Severity::Info,
            confidence: Confidence::Low,
            message,
            file: file.to_string(),
            start_line: Some(line),
            end_line: Some(line),
            remediation: meta.remediation.clone(),
            references: meta.references.clone(),
            limitations: vec![
                "Matching is by function name; tests in other crates are not seen.".into(),
                "A name match does not mean the transition is meaningfully tested.".into(),
            ],
        };

        if !any_test_code {
            let names: Vec<&str> = entrypoints.iter().map(|(_, n, _)| n.as_str()).collect();
            let shown = names.iter().take(5).copied().collect::<Vec<_>>().join(", ");
            let more = names.len().saturating_sub(5);
            let (file, _, line) = &entrypoints[0];
            return vec![finding(
                file,
                *line,
                format!(
                    "No test code was found in this project, but {} public function(s) change contract storage: {}{}. Consider tests for each state transition and for sequences of them.",
                    names.len(),
                    shown,
                    if more > 0 { format!(" (and {more} more)") } else { String::new() }
                ),
            )];
        }

        entrypoints
            .into_iter()
            .filter(|(_, name, _)| !tested.contains(name))
            .map(|(file, name, line)| {
                finding(
                    file,
                    line,
                    format!(
                        "Public function `{name}` changes contract storage, but no test code in this project mentions it. Consider tests for its authorized and unauthorized callers and for sequences with related transitions."
                    ),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(files: &[(&str, &str)]) -> Vec<Finding> {
        let parsed: Vec<(String, syn::File)> = files
            .iter()
            .map(|(p, s)| (p.to_string(), syn::parse_file(s).unwrap()))
            .collect();
        let units: Vec<SourceUnit<'_>> = parsed
            .iter()
            .map(|(p, a)| SourceUnit { path: p, ast: a })
            .collect();
        StateTransitionTestSuggestions.check_project(&ProjectUnit {
            files: &units,
            test_only: &std::collections::HashSet::new(),
        })
    }

    #[test]
    fn does_not_report_constructor() {
        let f = run(&[(
            "src/lib.rs",
            "use soroban_sdk::*; pub struct C; #[contractimpl] impl C { pub fn __constructor(env: Env) { env.storage().instance().set(&K, &1u32); } }",
        )]);
        assert!(f.is_empty(), "{f:?}");
    }

    const CONTRACT: &str = "use soroban_sdk::*; pub struct C; #[contractimpl] impl C {\n pub fn set_value(env: Env) { env.storage().instance().set(&K, &1u32); }\n pub fn other(env: Env) { env.storage().instance().remove(&K); }\n pub fn read(env: Env) -> u32 { env.storage().instance().get(&K).unwrap_or(0) } }";

    #[test]
    fn no_tests_yields_one_summary_finding() {
        let f = run(&[("src/lib.rs", CONTRACT)]);
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("No test code"));
        assert!(f[0].message.contains("set_value, other"));
        assert_eq!(f[0].start_line, Some(2));
    }

    #[test]
    fn tests_in_other_file_with_macro_calls_count() {
        let t =
            "#[test] fn t() { let c = Client::new(); assert_eq!(c.set_value(), ()); c.other(); }";
        assert!(run(&[("src/lib.rs", CONTRACT), ("src/test.rs", t)]).is_empty());
    }

    #[test]
    fn unreferenced_entrypoint_is_reported_individually() {
        let t = "#[test] fn t() { let c = Client::new(); c.set_value(); }";
        let f = run(&[("src/lib.rs", CONTRACT), ("src/test.rs", t)]);
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("`other`"));
        assert_eq!(f[0].file, "src/lib.rs");
    }

    #[test]
    fn inline_cfg_test_module_counts_but_definitions_do_not() {
        let src = format!("{CONTRACT}\n#[cfg(test)] mod tests {{ #[test] fn t() {{ let c = Client::new(); c.set_value(); }} }}");
        let f = run(&[("src/lib.rs", &src)]);
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("`other`"));
    }

    #[test]
    fn a_test_fn_in_the_contract_file_does_not_make_the_whole_file_test_code() {
        let src = format!("{CONTRACT}\n#[test] fn unrelated() {{ let _ = 1; }}");
        let f = run(&[("src/lib.rs", &src)]);
        assert_eq!(
            f.len(),
            2,
            "definitions in the contract file must not count as test mentions"
        );
    }

    #[test]
    fn read_only_contracts_have_nothing_to_suggest() {
        let src = "use soroban_sdk::*; pub struct C; #[contractimpl] impl C { pub fn read(env: Env) -> u32 { env.storage().instance().get(&K).unwrap_or(0) } }";
        assert!(run(&[("src/lib.rs", src)]).is_empty());
    }

    #[test]
    fn long_lists_are_truncated() {
        let fns: String = (0..7)
            .map(|i| {
                format!("pub fn f{i}(env: Env) {{ env.storage().instance().set(&K, &1u32); }}\n")
            })
            .collect();
        let src = format!("use soroban_sdk::*; pub struct C; #[contractimpl] impl C {{ {fns} }}");
        let f = run(&[("src/lib.rs", &src)]);
        assert!(f[0].message.contains("(and 2 more)"));
    }
}
