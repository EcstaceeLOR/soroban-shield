use crate::common::contract_entrypoints;
use proc_macro2::Span;
use sorobanshield_core::{Confidence, Finding, Rule, RuleMeta, RuleStatus, Severity, SourceUnit};
use std::collections::HashSet;
use syn::visit::Visit;
use syn::{Expr, ExprCall, ExprMethodCall, FnArg, Pat, Stmt, Type};

pub const ID: &str = "SS-003";

pub struct ExternalCallReview;

fn type_is_address(ty: &Type) -> bool {
    match ty {
        Type::Path(p) => p.path.segments.last().is_some_and(|s| s.ident == "Address"),
        Type::Reference(r) => type_is_address(&r.elem),
        _ => false,
    }
}

fn peel(mut e: &Expr) -> &Expr {
    loop {
        match e {
            Expr::Reference(r) => e = &r.expr,
            Expr::Paren(p) => e = &p.expr,
            _ => return e,
        }
    }
}

fn ident_of(e: &Expr) -> Option<String> {
    match peel(e) {
        Expr::Path(p) if p.path.segments.len() == 1 => Some(p.path.segments[0].ident.to_string()),
        _ => None,
    }
}

/// `SomeClient::new(..)` or `module::Client::new(..)`: a generated or SDK contract client.
fn client_new_args(e: &Expr) -> Option<&syn::punctuated::Punctuated<Expr, syn::Token![,]>> {
    let Expr::Call(ExprCall { func, args, .. }) = peel(e) else {
        return None;
    };
    let Expr::Path(p) = &**func else { return None };
    let segs = &p.path.segments;
    if segs.len() >= 2
        && segs[segs.len() - 1].ident == "new"
        && segs[segs.len() - 2].ident.to_string().ends_with("Client")
    {
        Some(args)
    } else {
        None
    }
}

struct Calls<'a> {
    addr_params: &'a HashSet<String>,
    clients: HashSet<String>,
    found: Vec<(Span, String)>,
}

impl Calls<'_> {
    fn is_client(&self, receiver: &Expr) -> bool {
        client_new_args(receiver).is_some()
            || ident_of(receiver).is_some_and(|n| self.clients.contains(&n))
    }

    fn discarded_try(&mut self, e: &Expr, at: Span) {
        if let Expr::MethodCall(m) = peel(e) {
            let name = m.method.to_string();
            let external = name == "try_invoke_contract"
                || (name.starts_with("try_") && self.is_client(&m.receiver));
            if external {
                self.found.push((
                    at,
                    format!("the Result of fallible external call `{name}` is discarded"),
                ));
            }
        }
    }
}

impl<'ast> Visit<'ast> for Calls<'_> {
    fn visit_stmt(&mut self, s: &'ast Stmt) {
        match s {
            Stmt::Local(l) => {
                if let Some(init) = &l.init {
                    if let (Pat::Ident(i), true) = (&l.pat, client_new_args(&init.expr).is_some()) {
                        self.clients.insert(i.ident.to_string());
                    }
                    if matches!(l.pat, Pat::Wild(_)) {
                        self.discarded_try(&init.expr, l.let_token.span);
                    }
                }
            }
            Stmt::Expr(e, Some(semi)) => self.discarded_try(e, semi.span),
            _ => {}
        }
        syn::visit::visit_stmt(self, s);
    }

    fn visit_expr_method_call(&mut self, m: &'ast ExprMethodCall) {
        let name = m.method.to_string();
        if (name == "invoke_contract" || name == "try_invoke_contract") && !m.args.is_empty() {
            if let Some(p) = ident_of(&m.args[0]).filter(|n| self.addr_params.contains(n)) {
                self.found.push((
                    m.method.span(),
                    format!("`{name}` targets the caller-supplied address `{p}`"),
                ));
            }
        }
        syn::visit::visit_expr_method_call(self, m);
    }

    fn visit_expr_call(&mut self, c: &'ast ExprCall) {
        let as_expr = Expr::Call(c.clone());
        if let Some(args) = client_new_args(&as_expr) {
            if let Some(p) = args
                .iter()
                .skip(1)
                .find_map(|a| ident_of(a).filter(|n| self.addr_params.contains(n)))
            {
                let span = match &*c.func {
                    Expr::Path(path) => path.path.segments.last().map(|s| s.ident.span()),
                    _ => None,
                }
                .unwrap_or_else(Span::call_site);
                self.found.push((
                    span,
                    format!("a contract client is created for the caller-supplied address `{p}`"),
                ));
            }
        }
        syn::visit::visit_expr_call(self, c);
    }
}

impl Rule for ExternalCallReview {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: ID.into(),
            name: "External contract call review".into(),
            status: RuleStatus::Implemented,
            severity: Some(Severity::Low),
            confidence: Some(Confidence::Low),
            concern: "A public contract function either calls a contract whose address is taken directly from a caller-supplied Address argument, or discards the Result of a fallible external call. A call by itself is not a vulnerability; these two shapes are where the callee's identity or a failure is easy to overlook.".into(),
            detection: "Parses each file with syn. In public functions of #[contractimpl] impl blocks it reports (a) invoke_contract / try_invoke_contract whose first argument is a function parameter of type Address, and `<Name>Client::new(env, addr)` / `module::Client::new(env, addr)` constructed from such a parameter; and (b) `try_*` calls made on a value known to be a contract client (bound from `...Client::new(..)` in the same function) or `try_invoke_contract`, when the Result is discarded with `let _ =` or used as an expression statement. #[cfg(test)] items are skipped.".into(),
            severity_rationale: "Low severity and low confidence: calling a caller-chosen contract can be intended (routers, aggregators), and Soroban's host rejects re-entrancy and reverts all state when an invocation fails, so the typical consequence is a surprising call target or an ignored failure rather than a classic drain. The rule cannot see whether the address is validated against an allow-list or whether ignoring the failure is deliberate.".into(),
            remediation: "If the target must be trusted, compare the address against a stored allow-list or configured address before calling it, and require_auth the actor who selects it. Handle the Result of try_* calls explicitly (propagate with `?`, map to a contract error, or document why ignoring it is safe). Add tests with a malicious or failing callee.".into(),
            false_positives: vec![
                "Routers and generic proxies that intentionally call a caller-chosen contract.".into(),
                "The address was validated earlier in the function; validation is not evaluated.".into(),
                "Intentionally best-effort calls whose failure is acceptable.".into(),
            ],
            false_negatives: vec![
                "Targets reached through locals, struct fields, storage reads or helper functions: only direct Address parameters are tracked.".into(),
                "try_* calls on clients obtained other than by `...Client::new(..)` in the same function, and discarded results via helper wrappers.".into(),
                "External calls in private helpers, other files or macros, and calls whose Result is handled incorrectly rather than discarded.".into(),
                "State written before or after an external call; ordering is not analysed.".into(),
            ],
            references: vec![
                "https://developers.stellar.org/docs/build/smart-contracts/example-contracts/cross-contract-call".into(),
                "https://developers.stellar.org/docs/build/smart-contracts".into(),
            ],
            fixtures: vec![
                "tests/fixtures/positive/external_calls".into(),
                "tests/fixtures/negative/safe_external_calls".into(),
            ],
        }
    }

    fn check(&self, unit: &SourceUnit<'_>) -> Vec<Finding> {
        let meta = self.meta();
        let mut out = Vec::new();
        for f in contract_entrypoints(&unit.ast.items) {
            let addr_params: HashSet<String> = f
                .sig
                .inputs
                .iter()
                .filter_map(|a| match a {
                    FnArg::Typed(t) if type_is_address(&t.ty) => match &*t.pat {
                        Pat::Ident(i) => Some(i.ident.to_string()),
                        _ => None,
                    },
                    _ => None,
                })
                .collect();
            let mut calls = Calls {
                addr_params: &addr_params,
                clients: HashSet::new(),
                found: Vec::new(),
            };
            calls.visit_block(&f.block);
            if calls.found.is_empty() {
                continue;
            }
            let first = calls
                .found
                .iter()
                .map(|(s, _)| s.start().line)
                .min()
                .unwrap_or(0);
            let last = calls
                .found
                .iter()
                .map(|(s, _)| s.end().line)
                .max()
                .unwrap_or(first);
            let details = calls
                .found
                .iter()
                .map(|(_, d)| d.as_str())
                .take(3)
                .collect::<Vec<_>>()
                .join("; ");
            out.push(Finding {
                rule_id: ID.into(),
                title: meta.name.clone(),
                severity: Severity::Low,
                confidence: Confidence::Low,
                message: format!(
                    "Public contract function `{}` has {} external-call site(s) worth reviewing: {}. Whether this is a problem depends on validation and intent that this rule cannot see.",
                    f.sig.ident,
                    calls.found.len(),
                    details
                ),
                file: unit.path.to_string(),
                start_line: Some(first),
                end_line: Some(last),
                remediation: meta.remediation.clone(),
                references: meta.references.clone(),
                limitations: vec![
                    "Only direct Address parameters and same-function client bindings are tracked.".into(),
                    "Address validation and the intent of ignoring a failure are not evaluated.".into(),
                ],
            });
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(body: &str) -> Vec<Finding> {
        let src = format!("use soroban_sdk::*; pub struct C; #[contractimpl] impl C {{ {body} }}");
        let ast = syn::parse_file(&src).unwrap();
        ExternalCallReview.check(&SourceUnit {
            path: "lib.rs",
            ast: &ast,
        })
    }

    #[test]
    fn flags_invoke_contract_on_address_parameter() {
        let f = run("pub fn go(env: Env, target: Address, f: Symbol) { env.invoke_contract::<()>(&target, &f, vec![&env]); }");
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("caller-supplied address `target`"));
    }

    #[test]
    fn flags_client_built_from_address_parameter() {
        let f = run("pub fn go(env: Env, token: Address) { let c = token::Client::new(&env, &token); c.balance(&token); }");
        assert_eq!(f.len(), 1);
        let g = run("pub fn go(env: Env, t: Address) { OtherClient::new(&env, &t).ping(); }");
        assert_eq!(g.len(), 1);
    }

    #[test]
    fn address_from_storage_or_local_is_not_flagged() {
        assert!(run("pub fn go(env: Env) { let a: Address = env.storage().instance().get(&K).unwrap(); let c = token::Client::new(&env, &a); c.balance(&a); }").is_empty());
    }

    #[test]
    fn non_address_parameter_is_not_flagged() {
        assert!(
            run("pub fn go(env: Env, id: u32) { let c = token::Client::new(&env, &id); }")
                .is_empty()
        );
    }

    #[test]
    fn flags_discarded_try_result_on_known_client() {
        assert_eq!(run("pub fn go(env: Env, a: u32) { let c = Peer::Client::new(&env, &env.current_contract_address()); let _ = c.try_ping(); }").len(), 1);
        assert_eq!(run("pub fn go(env: Env) { let c = PeerClient::new(&env, &env.current_contract_address()); c.try_ping(); }").len(), 1);
        assert_eq!(run("pub fn go(env: Env) { let _ = env.try_invoke_contract::<(), Error>(&env.current_contract_address(), &Symbol::new(&env, \"x\"), vec![&env]); }").len(), 1);
    }

    #[test]
    fn handled_try_results_and_std_try_methods_are_not_flagged() {
        assert!(run("pub fn go(env: Env) -> bool { let c = PeerClient::new(&env, &env.current_contract_address()); c.try_ping().is_ok() }").is_empty());
        assert!(run("pub fn go(env: Env) { let c = PeerClient::new(&env, &env.current_contract_address()); let r = c.try_ping(); drop(r); }").is_empty());
        assert!(run("pub fn go(x: u64) { let _ = u32::try_from(x); x.try_into(); }").is_empty());
    }

    #[test]
    fn skips_private_and_cfg_test_code() {
        assert!(run("fn go(env: Env, t: Address, f: Symbol) { env.invoke_contract::<()>(&t, &f, vec![&env]); }").is_empty());
        let ast = syn::parse_file("#[cfg(test)] mod t { pub struct C; #[contractimpl] impl C { pub fn go(env: Env, t: Address, f: Symbol) { env.invoke_contract::<()>(&t, &f, vec![&env]); } } }").unwrap();
        assert!(ExternalCallReview
            .check(&SourceUnit {
                path: "x.rs",
                ast: &ast
            })
            .is_empty());
    }

    #[test]
    fn one_finding_per_function() {
        let f = run("pub fn go(env: Env, a: Address, b: Address, f: Symbol) { env.invoke_contract::<()>(&a, &f, vec![&env]); env.invoke_contract::<()>(&b, &f, vec![&env]); }");
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("2 external-call"));
    }
}
