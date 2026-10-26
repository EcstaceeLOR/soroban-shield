use crate::common::{has_attr, is_cfg_test, receiver_chain_has_storage, WRITE_CALLS};
use proc_macro2::Span;
use sorobanshield_core::{Confidence, Finding, Rule, RuleMeta, RuleStatus, Severity, SourceUnit};
use std::collections::HashSet;
use syn::visit::Visit;
use syn::{Expr, ExprCall, ExprMethodCall, ImplItem, Item, ItemFn, ItemImpl, ItemMod, Visibility};

pub const ID: &str = "SS-001";

pub struct AuthorizationPathReview;

const AUTH_CALLS: &[&str] = &["require_auth", "require_auth_for_args"];

#[derive(Default)]
struct BodyFacts {
    has_auth: bool,
    callees: HashSet<String>,
    writes: Vec<Span>,
}

impl<'ast> Visit<'ast> for BodyFacts {
    fn visit_expr_method_call(&mut self, m: &'ast ExprMethodCall) {
        let name = m.method.to_string();
        if AUTH_CALLS.contains(&name.as_str()) {
            self.has_auth = true;
        } else if WRITE_CALLS.contains(&name.as_str()) && receiver_chain_has_storage(&m.receiver) {
            self.writes.push(m.method.span());
        }
        self.callees.insert(name);
        syn::visit::visit_expr_method_call(self, m);
    }

    fn visit_expr_call(&mut self, c: &'ast ExprCall) {
        if let Expr::Path(p) = &*c.func {
            if let Some(seg) = p.path.segments.last() {
                let name = seg.ident.to_string();
                if AUTH_CALLS.contains(&name.as_str()) {
                    self.has_auth = true;
                }
                self.callees.insert(name);
            }
        }
        syn::visit::visit_expr_call(self, c);
    }
}

fn facts(block: &syn::Block) -> BodyFacts {
    let mut f = BodyFacts::default();
    f.visit_block(block);
    f
}

struct Collector<'a> {
    /// Names of functions in this file that directly call require_auth.
    auth_helpers: HashSet<String>,
    entrypoints: Vec<(&'a syn::Signature, &'a syn::Block)>,
}

impl<'a> Collector<'a> {
    fn items(&mut self, items: &'a [Item]) {
        for item in items {
            match item {
                Item::Fn(ItemFn {
                    attrs, sig, block, ..
                }) if !is_cfg_test(attrs) => {
                    self.note_helper(&sig.ident.to_string(), block);
                }
                Item::Impl(ItemImpl { attrs, items, .. }) if !is_cfg_test(attrs) => {
                    let contract = has_attr(attrs, "contractimpl");
                    for it in items {
                        if let ImplItem::Fn(f) = it {
                            self.note_helper(&f.sig.ident.to_string(), &f.block);
                            if contract && matches!(f.vis, Visibility::Public(_)) {
                                self.entrypoints.push((&f.sig, &f.block));
                            }
                        }
                    }
                }
                Item::Mod(ItemMod {
                    attrs,
                    content: Some((_, inner)),
                    ..
                }) if !is_cfg_test(attrs) => {
                    self.items(inner);
                }
                _ => {}
            }
        }
    }

    fn note_helper(&mut self, name: &str, block: &syn::Block) {
        if facts(block).has_auth {
            self.auth_helpers.insert(name.to_string());
        }
    }
}

impl Rule for AuthorizationPathReview {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: ID.into(),
            name: "Authorization path review".into(),
            status: RuleStatus::Implemented,
            severity: Some(Severity::Medium),
            confidence: Some(Confidence::Low),
            concern: "A public contract function writes to contract storage, but no require_auth call is visible in the function or in a same-file helper it calls. Authorization may be missing, or it may be enforced somewhere this rule cannot see.".into(),
            detection: "Parses each Rust file with syn. In impl blocks annotated #[contractimpl], for each public function it looks for storage writes (a .set(..) or .remove(..) call whose receiver chain contains .storage()) and for require_auth / require_auth_for_args calls. A function is reported when it writes storage, contains no such auth call, and calls no same-file function that does. Items under #[cfg(test)] are skipped.".into(),
            severity_rationale: "Medium: missing authorization on a storage write can be serious, but this rule only establishes that no auth call is visible, not that authorization is absent. Confidence is low for the same reason, so the finding is a prompt for manual review, not a vulnerability claim.".into(),
            remediation: "Trace every caller of the function. If the write must be restricted, call require_auth() on the relevant Address before it, and add tests for authorized and unauthorized callers. If authorization is enforced elsewhere (for example the function is only reachable through an authorized entrypoint), document that and treat the finding as reviewed.".into(),
            false_positives: vec![
                "Public initialization functions that are intentionally callable once by anyone.".into(),
                "Authorization performed in a helper defined in another file or crate, in a caller, or via a trait method.".into(),
                "Writes to temporary storage that carry no security meaning.".into(),
            ],
            false_negatives: vec![
                "A require_auth call exists but is on the wrong Address, or is conditional and skipped on some paths.".into(),
                "Storage writes hidden behind macros, helper functions in other files, or non-set/remove APIs (for example extend_ttl or bump).".into(),
                "Functions not marked pub or not inside #[contractimpl], and contract code that is generated rather than written.".into(),
                "Any call named require_auth on an unrelated type is counted as authorization.".into(),
            ],
            references: vec![
                "https://developers.stellar.org/docs/build/smart-contracts/example-contracts/auth".into(),
                "https://docs.rs/soroban-sdk/latest/soroban_sdk/struct.Address.html#method.require_auth".into(),
            ],
            fixtures: vec![
                "tests/fixtures/positive/unprotected_write".into(),
                "tests/fixtures/negative/protected_write".into(),
                "tests/fixtures/negative/helper_auth".into(),
                "tests/fixtures/edge/test_module_ignored".into(),
            ],
        }
    }

    fn check(&self, unit: &SourceUnit<'_>) -> Vec<Finding> {
        let mut c = Collector {
            auth_helpers: HashSet::new(),
            entrypoints: Vec::new(),
        };
        c.items(&unit.ast.items);
        let meta = self.meta();

        let mut out = Vec::new();
        for (sig, block) in &c.entrypoints {
            let f = facts(block);
            if f.writes.is_empty() || f.has_auth {
                continue;
            }
            if f.callees
                .iter()
                .any(|n| c.auth_helpers.contains(n) && sig.ident != n.as_str())
            {
                continue;
            }
            let first = f.writes[0].start().line;
            let last = f.writes.last().map(|s| s.end().line).unwrap_or(first);
            out.push(Finding {
                rule_id: ID.into(),
                title: meta.name.clone(),
                severity: Severity::Medium,
                confidence: Confidence::Low,
                message: format!(
                    "Public contract function `{}` writes to storage ({} write(s)) but no require_auth call is visible in it or in a same-file helper it calls. Review whether every caller is meant to be allowed to do this.",
                    sig.ident,
                    f.writes.len()
                ),
                file: unit.path.to_string(),
                start_line: Some(first),
                end_line: Some(last),
                remediation: meta.remediation.clone(),
                references: meta.references.clone(),
                limitations: vec![
                    "Authorization enforced in another file, a caller or a trait method is not visible to this rule.".into(),
                    "This is a review prompt, not proof that authorization is missing.".into(),
                ],
            });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> Vec<Finding> {
        let ast = syn::parse_file(src).unwrap();
        AuthorizationPathReview.check(&SourceUnit {
            path: "lib.rs",
            ast: &ast,
        })
    }

    const HEAD: &str = "use soroban_sdk::*; pub struct C; #[contractimpl] impl C {";

    #[test]
    fn flags_unprotected_storage_write() {
        let f = run(&format!(
            "{HEAD} pub fn put(env: Env, v: u32) {{ env.storage().instance().set(&KEY, &v); }} }}"
        ));
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].rule_id, "SS-001");
        assert_eq!(f[0].start_line, Some(1));
    }

    #[test]
    fn flags_remove_on_persistent_storage() {
        let f = run(&format!(
            "{HEAD} pub fn rm(env: Env) {{ env.storage().persistent().remove(&KEY); }} }}"
        ));
        assert_eq!(f.len(), 1);
    }

    #[test]
    fn accepts_direct_require_auth() {
        let f = run(&format!("{HEAD} pub fn put(env: Env, who: Address) {{ who.require_auth(); env.storage().instance().set(&KEY, &1u32); }} }}"));
        assert!(f.is_empty());
    }

    #[test]
    fn accepts_require_auth_for_args() {
        let f = run(&format!("{HEAD} pub fn put(env: Env, who: Address) {{ who.require_auth_for_args(vec![&env]); env.storage().instance().set(&KEY, &1u32); }} }}"));
        assert!(f.is_empty());
    }

    #[test]
    fn accepts_same_file_auth_helper() {
        let f = run(&format!("fn guard(a: &Address) {{ a.require_auth(); }} {HEAD} pub fn put(env: Env, who: Address) {{ guard(&who); env.storage().instance().set(&KEY, &1u32); }} }}"));
        assert!(f.is_empty());
    }

    #[test]
    fn auth_helper_that_is_a_method_is_recognised() {
        let f = run(&format!("{HEAD} fn guard(who: &Address) {{ who.require_auth(); }} pub fn put(env: Env, who: Address) {{ Self::guard(&who); env.storage().instance().set(&KEY, &1u32); }} }}"));
        assert!(f.is_empty());
    }

    #[test]
    fn ignores_reads_and_unrelated_set_calls() {
        let f = run(&format!("{HEAD} pub fn get(env: Env) -> u32 {{ let mut m = Map::new(&env); m.set(1, 2); env.storage().instance().get(&KEY).unwrap() }} }}"));
        assert!(f.is_empty());
    }

    #[test]
    fn ignores_non_public_and_non_contract_functions() {
        let f = run(&format!("{HEAD} fn private(env: Env) {{ env.storage().instance().set(&KEY, &1u32); }} }} pub struct D; impl D {{ pub fn x(env: Env) {{ env.storage().instance().set(&KEY, &1u32); }} }}"));
        assert!(f.is_empty());
    }

    #[test]
    fn skips_cfg_test_modules() {
        let f = run("#[cfg(test)] mod tests { use soroban_sdk::*; pub struct C; #[contractimpl] impl C { pub fn put(env: Env) { env.storage().instance().set(&K, &1u32); } } }");
        assert!(f.is_empty());
    }

    #[test]
    fn one_finding_per_function_not_per_write() {
        let f = run(&format!("{HEAD} pub fn put(env: Env) {{ env.storage().instance().set(&A, &1u32); env.storage().instance().set(&B, &2u32); }} }}"));
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("2 write"));
    }
}
