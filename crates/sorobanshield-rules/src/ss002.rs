use crate::common::{contract_entrypoints, is_cfg_test};
use proc_macro2::Span;
use sorobanshield_core::{Confidence, Finding, Rule, RuleMeta, RuleStatus, Severity, SourceUnit};
use std::collections::{HashMap, HashSet};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    BinOp, Expr, ExprBinary, ExprCall, ExprCast, ExprMethodCall, FnArg, ImplItem, Item, ItemFn,
    ItemImpl, ItemMod, Local, Pat, Type,
};

pub const ID: &str = "SS-002";

pub struct ArithmeticBoundaryReview;

/// (bits, signed) for the primitive integer types this rule understands.
fn int_kind(name: &str) -> Option<(u32, bool)> {
    Some(match name {
        "i32" => (32, true),
        "u32" => (32, false),
        "i64" => (64, true),
        "u64" => (64, false),
        "i128" => (128, true),
        "u128" => (128, false),
        _ => return None,
    })
}

fn type_kind(ty: &Type) -> Option<(u32, bool)> {
    match ty {
        Type::Path(p) if p.qself.is_none() && p.path.segments.len() == 1 => {
            int_kind(&p.path.segments[0].ident.to_string())
        }
        _ => None,
    }
}

fn single_ident(e: &Expr) -> Option<String> {
    match e {
        Expr::Path(p) if p.path.segments.len() == 1 => Some(p.path.segments[0].ident.to_string()),
        Expr::Paren(p) => single_ident(&p.expr),
        Expr::Group(g) => single_ident(&g.expr),
        _ => None,
    }
}

/// A tracked integer value: its kind and the parameter it derives from.
#[derive(Clone)]
struct Tracked {
    kind: (u32, bool),
    origin: String,
}

type Env = HashMap<String, Tracked>;

/// Tracked integer parameters of a function, optionally restricted to the
/// positions (counting a receiver as position 0) that callers pass tracked values in.
fn tracked_params(sig: &syn::Signature, only: Option<&HashSet<usize>>) -> Env {
    let mut env = Env::new();
    for (i, a) in sig.inputs.iter().enumerate() {
        if only.is_some_and(|o| !o.contains(&i)) {
            continue;
        }
        if let FnArg::Typed(t) = a {
            if let (Pat::Ident(id), Some(kind)) = (&*t.pat, type_kind(&t.ty)) {
                let name = id.ident.to_string();
                env.insert(name.clone(), Tracked { kind, origin: name });
            }
        }
    }
    env
}

/// A call to a function in the same file that receives a tracked value.
struct CallOut {
    callee: String,
    /// Positions in the callee's `inputs` (receiver counted) that get a tracked value.
    positions: HashSet<usize>,
}

struct Walker {
    env: Env,
    found: Vec<(Span, String)>,
    calls: Vec<CallOut>,
}

impl Walker {
    /// The tracked value an expression carries, if it is a tracked name, or
    /// arithmetic or a cast over one.
    fn derived(&self, e: &Expr) -> Option<Tracked> {
        match e {
            Expr::Paren(p) => self.derived(&p.expr),
            Expr::Group(g) => self.derived(&g.expr),
            Expr::Reference(r) => self.derived(&r.expr),
            Expr::Path(_) => single_ident(e).and_then(|n| self.env.get(&n).cloned()),
            Expr::Binary(b) if matches!(b.op, BinOp::Add(_) | BinOp::Sub(_) | BinOp::Mul(_)) => {
                self.derived(&b.left).or_else(|| self.derived(&b.right))
            }
            Expr::Cast(c) => {
                let from = self.derived(&c.expr)?;
                type_kind(&c.ty).map(|kind| Tracked {
                    kind,
                    origin: from.origin,
                })
            }
            _ => None,
        }
    }

    fn describe(&self, name: &str) -> String {
        match self.env.get(name) {
            Some(t) if t.origin != name => {
                format!("`{name}` (derived from integer parameter `{}`)", t.origin)
            }
            _ => format!("integer parameter `{name}`"),
        }
    }

    fn call(&mut self, callee: String, args: &[&Expr], receiver_offset: usize) {
        let positions: HashSet<usize> = args
            .iter()
            .enumerate()
            .filter(|(_, a)| self.derived(a).is_some())
            .map(|(i, _)| i + receiver_offset)
            .collect();
        if !positions.is_empty() {
            self.calls.push(CallOut { callee, positions });
        }
    }
}

impl<'ast> Visit<'ast> for Walker {
    fn visit_expr_binary(&mut self, b: &'ast ExprBinary) {
        let op = match b.op {
            BinOp::Add(_) | BinOp::AddAssign(_) => Some("+"),
            BinOp::Sub(_) | BinOp::SubAssign(_) => Some("-"),
            BinOp::Mul(_) | BinOp::MulAssign(_) => Some("*"),
            _ => None,
        };
        if let Some(op) = op {
            let name = [&b.left, &b.right]
                .into_iter()
                .filter_map(|s| single_ident(s))
                .find(|n| self.env.contains_key(n));
            if let Some(name) = name {
                self.found.push((
                    b.op.span(),
                    format!("unchecked `{op}` on {}", self.describe(&name)),
                ));
            }
        }
        syn::visit::visit_expr_binary(self, b);
    }

    fn visit_expr_cast(&mut self, c: &'ast ExprCast) {
        if let (Some(name), Some((to_bits, to_signed))) = (single_ident(&c.expr), type_kind(&c.ty))
        {
            if let Some(t) = self.env.get(&name) {
                let (from_bits, from_signed) = t.kind;
                if to_bits < from_bits || (to_bits == from_bits && to_signed != from_signed) {
                    self.found.push((
                        c.as_token.span,
                        format!(
                            "`as` cast of {} may truncate or change sign",
                            self.describe(&name)
                        ),
                    ));
                }
            }
        }
        syn::visit::visit_expr_cast(self, c);
    }

    fn visit_local(&mut self, l: &'ast Local) {
        // Visit the initializer first so its own sites and calls are seen with
        // the bindings that existed before this `let`.
        if let Some(init) = &l.init {
            self.visit_expr(&init.expr);
            if let Some((_, diverge)) = &init.diverge {
                self.visit_expr(diverge);
            }
        }
        let (name, annotated) = match &l.pat {
            Pat::Ident(i) => (Some(i.ident.to_string()), None),
            Pat::Type(t) => match &*t.pat {
                Pat::Ident(i) => (Some(i.ident.to_string()), type_kind(&t.ty)),
                _ => (None, None),
            },
            _ => (None, None),
        };
        let Some(name) = name else { return };
        let derived = l.init.as_ref().and_then(|i| self.derived(&i.expr));
        match derived {
            Some(t) => {
                self.env.insert(
                    name,
                    Tracked {
                        kind: annotated.unwrap_or(t.kind),
                        origin: t.origin,
                    },
                );
            }
            // A new binding of the same name that is not tracked shadows the old one.
            None => {
                self.env.remove(&name);
            }
        }
    }

    fn visit_expr_call(&mut self, c: &'ast ExprCall) {
        if let Expr::Path(p) = &*c.func {
            if let Some(seg) = p.path.segments.last() {
                let args: Vec<&Expr> = c.args.iter().collect();
                self.call(seg.ident.to_string(), &args, 0);
            }
        }
        syn::visit::visit_expr_call(self, c);
    }

    fn visit_expr_method_call(&mut self, m: &'ast ExprMethodCall) {
        if single_ident(&m.receiver).is_some_and(|r| r == "self") {
            let args: Vec<&Expr> = m.args.iter().collect();
            self.call(m.method.to_string(), &args, 1);
        }
        syn::visit::visit_expr_method_call(self, m);
    }
}

struct FnDef<'a> {
    sig: &'a syn::Signature,
    block: &'a syn::Block,
}

/// Every function body in the file that is not under `#[cfg(test)]`, by name.
fn collect_fns<'a>(items: &'a [Item], out: &mut HashMap<String, Vec<FnDef<'a>>>) {
    for item in items {
        match item {
            Item::Fn(ItemFn {
                attrs, sig, block, ..
            }) if !is_cfg_test(attrs) => out
                .entry(sig.ident.to_string())
                .or_default()
                .push(FnDef { sig, block }),
            Item::Impl(ItemImpl { attrs, items, .. }) if !is_cfg_test(attrs) => {
                for it in items {
                    if let ImplItem::Fn(f) = it {
                        out.entry(f.sig.ident.to_string()).or_default().push(FnDef {
                            sig: &f.sig,
                            block: &f.block,
                        });
                    }
                }
            }
            Item::Mod(ItemMod {
                attrs,
                content: Some((_, inner)),
                ..
            }) if !is_cfg_test(attrs) => collect_fns(inner, out),
            _ => {}
        }
    }
}

fn walk(env: Env, block: &syn::Block) -> Walker {
    let mut w = Walker {
        env,
        found: Vec::new(),
        calls: Vec::new(),
    };
    w.visit_block(block);
    w
}

fn summarize(found: &[(Span, String)]) -> (usize, usize, String, usize) {
    let first = found.iter().map(|(s, _)| s.start().line).min().unwrap_or(0);
    let last = found
        .iter()
        .map(|(s, _)| s.end().line)
        .max()
        .unwrap_or(first);
    let detail = found
        .iter()
        .map(|(_, d)| d.as_str())
        .take(3)
        .collect::<Vec<_>>()
        .join("; ");
    (first, last, detail, found.len().saturating_sub(3))
}

impl Rule for ArithmeticBoundaryReview {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: ID.into(),
            name: "Arithmetic boundary review".into(),
            status: RuleStatus::Implemented,
            severity: Some(Severity::Low),
            confidence: Some(Confidence::Low),
            concern: "Public contract functions (and same-file helpers they pass integer arguments to) that apply +, - or * (or their assignment forms) to an integer argument or a local derived from one, or cast such a value to a narrower or differently signed type, may overflow, underflow, truncate or change sign on attacker-chosen input. Whether that is a problem depends on the build's overflow-checks setting and on validation this rule cannot see.".into(),
            detection: "Parses each file with syn. In public functions of #[contractimpl] impl blocks it records parameters whose declared type is one of i32, u32, i64, u64, i128 or u128. It reports a function when the body applies +, -, *, +=, -= or *= with such a parameter as a direct operand, or uses `param as T` where T is narrower than the parameter type or has the same width but different signedness. A local introduced by `let` is tracked when its initializer is a tracked name, arithmetic (+, -, *) over one, or a cast of one; a later `let` of the same name that is not tracked stops the tracking. When a tracked value is passed as an argument to a function of the same file whose name is defined exactly once (and is not itself a contract entrypoint), the matching parameter of that helper is tracked too, through `f(..)`, `Self::f(..)` and `self.f(..)` calls, and a finding is reported in the helper at the arithmetic site. Calls such as checked_add, saturating_sub or try_into are method calls and are never reported. #[cfg(test)] items are skipped.".into(),
            severity_rationale: "Low severity: with overflow-checks enabled (as Soroban contract release profiles commonly do), overflow traps instead of wrapping, so the common outcome is a failed invocation rather than corrupted state; truncating casts and builds without overflow checks are worse. Low confidence: only direct uses of parameters are visible, and domain constraints that bound the values are not understood. This is a review prompt, not a vulnerability claim.".into(),
            remediation: "Decide what should happen at the boundary. Use checked_add / checked_sub / checked_mul and return a contract error, or saturating/wrapping variants if that is intended. Replace narrowing `as` casts with try_into and handle the error. Validate argument ranges, confirm overflow-checks = true in the release profile, and add tests at the numeric boundaries.".into(),
            false_positives: vec![
                "Arithmetic on values already bounded by earlier validation or by the domain (for example a small loop counter parameter).".into(),
                "Builds with overflow-checks enabled, where overflow traps deterministically and that is acceptable.".into(),
                "Intentional wrapping or truncating behavior.".into(),
            ],
            false_negatives: vec![
                "Arithmetic on struct fields, storage reads, function results, or locals that are not derived from a tracked integer by a plain `let` (for example through a tuple pattern, a loop, or a method call such as clone or into).".into(),
                "Operations inside helpers in other files, helpers whose name is defined more than once in the file, calls through method receivers other than self, and macros. Flow is not tracked: a local keeps its tracking in every branch, so a value reassigned to a bounded constant on one path is still treated as tracked.".into(),
                "Type aliases for integers and i8/i16/u8/u16/usize/isize parameters, which are not recognised.".into(),
                "Division and remainder, shifts, and casts of non-parameter expressions.".into(),
                "Rounding and precision problems in fixed-point math.".into(),
            ],
            references: vec![
                "https://doc.rust-lang.org/book/ch03-02-data-types.html#integer-overflow".into(),
                "https://developers.stellar.org/docs/build/smart-contracts/overview".into(),
            ],
            fixtures: vec![
                "tests/fixtures/positive/unchecked_arithmetic".into(),
                "tests/fixtures/negative/checked_arithmetic".into(),
                "tests/fixtures/positive/local_arithmetic".into(),
                "tests/fixtures/positive/helper_arithmetic".into(),
                "tests/fixtures/edge/storage_arithmetic_untracked".into(),
            ],
        }
    }

    fn check(&self, unit: &SourceUnit<'_>) -> Vec<Finding> {
        let meta = self.meta();
        let entrypoints = contract_entrypoints(&unit.ast.items);
        let entry_names: HashSet<String> = entrypoints
            .iter()
            .map(|f| f.sig.ident.to_string())
            .collect();
        let mut table: HashMap<String, Vec<FnDef<'_>>> = HashMap::new();
        collect_fns(&unit.ast.items, &mut table);

        let finding = |message: String, found: &[(Span, String)], limitations: Vec<String>| {
            let (first, last, _, _) = summarize(found);
            Finding {
                rule_id: ID.into(),
                title: meta.name.clone(),
                severity: Severity::Low,
                confidence: Confidence::Low,
                message,
                file: unit.path.to_string(),
                start_line: Some(first),
                end_line: Some(last),
                remediation: meta.remediation.clone(),
                references: meta.references.clone(),
                limitations,
            }
        };
        let base_limits = || {
            vec![
                "Only integer parameters, locals derived from them, and same-file helpers that receive them are tracked; fields and storage values are not.".to_string(),
                "Overflow-checks settings and surrounding validation are not evaluated.".to_string(),
            ]
        };

        let mut out = Vec::new();
        // Helper name -> (tracked positions, first calling entrypoint).
        let mut helpers: HashMap<String, (HashSet<usize>, String)> = HashMap::new();
        let mut queue: Vec<(String, CallOut)> = Vec::new();

        for f in &entrypoints {
            let w = walk(tracked_params(&f.sig, None), &f.block);
            let entry = f.sig.ident.to_string();
            queue.extend(w.calls.into_iter().map(|c| (entry.clone(), c)));
            if w.found.is_empty() {
                continue;
            }
            let (_, _, detail, more) = summarize(&w.found);
            out.push(finding(
                format!(
                    "Public contract function `{}` has {} arithmetic or cast site(s) on integer arguments that deserve a boundary review: {}{}. Whether this can overflow depends on overflow-checks and on validation not visible to this rule.",
                    f.sig.ident,
                    w.found.len(),
                    detail,
                    if more > 0 { format!(" (and {more} more)") } else { String::new() }
                ),
                &w.found,
                base_limits(),
            ));
        }

        // Follow tracked values into same-file helpers until no new position
        // is reached. A helper is only followed when its name is unique in the
        // file and is not itself an entrypoint.
        let mut rounds = 0;
        while let Some((entry, call)) = queue.pop() {
            rounds += 1;
            if rounds > 200 {
                break;
            }
            let unique = table
                .get(&call.callee)
                .is_some_and(|d| d.len() == 1 && !entry_names.contains(&call.callee));
            if !unique {
                continue;
            }
            let def = &table[&call.callee][0];
            let known = helpers
                .entry(call.callee.clone())
                .or_insert_with(|| (HashSet::new(), entry.clone()));
            let before = known.0.len();
            known.0.extend(call.positions.iter().copied());
            if known.0.len() == before && before != 0 {
                continue;
            }
            let w = walk(tracked_params(def.sig, Some(&known.0)), def.block);
            queue.extend(w.calls.into_iter().map(|c| (entry.clone(), c)));
        }

        let mut names: Vec<&String> = helpers.keys().collect();
        names.sort();
        for name in names {
            let (positions, entry) = &helpers[name];
            let def = &table[name][0];
            let w = walk(tracked_params(def.sig, Some(positions)), def.block);
            if w.found.is_empty() {
                continue;
            }
            let (_, _, detail, more) = summarize(&w.found);
            let mut limits = vec![format!(
                "Reached from public contract function `{entry}` by a same-file call matched by name; the value passed may be bounded by validation this rule does not see."
            )];
            limits.extend(base_limits());
            out.push(finding(
                format!(
                    "Helper `{name}`, called from public contract function `{entry}` with integer arguments, has {} arithmetic or cast site(s) that deserve a boundary review: {}{}. Whether this can overflow depends on overflow-checks and on validation not visible to this rule.",
                    w.found.len(),
                    detail,
                    if more > 0 { format!(" (and {more} more)") } else { String::new() }
                ),
                &w.found,
                limits,
            ));
        }
        out.sort_by_key(|f| f.sort_key());
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(body: &str) -> Vec<Finding> {
        let src = format!("use soroban_sdk::*; pub struct C; #[contractimpl] impl C {{ {body} }}");
        let ast = syn::parse_file(&src).unwrap();
        ArithmeticBoundaryReview.check(&SourceUnit {
            path: "lib.rs",
            ast: &ast,
        })
    }

    fn run_file(src: &str) -> Vec<Finding> {
        let src = format!("use soroban_sdk::*; {src}");
        let ast = syn::parse_file(&src).unwrap();
        ArithmeticBoundaryReview.check(&SourceUnit {
            path: "lib.rs",
            ast: &ast,
        })
    }

    #[test]
    fn flags_add_on_parameter() {
        let f = run("pub fn f(a: i128, b: i128) -> i128 { a + b }");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].rule_id, "SS-002");
        assert_eq!(f[0].start_line, Some(1));
    }

    #[test]
    fn flags_compound_assignment_and_mul_sub() {
        assert_eq!(
            run("pub fn f(mut a: u64, b: u64) -> u64 { a += b; a }").len(),
            1
        );
        assert_eq!(run("pub fn f(a: u64, b: u64) -> u64 { a - b }").len(), 1);
        assert_eq!(run("pub fn f(a: u32) -> u32 { 2 * a }").len(), 1);
    }

    #[test]
    fn flags_narrowing_and_sign_changing_casts() {
        assert_eq!(run("pub fn f(a: u128) -> u64 { a as u64 }").len(), 1);
        assert_eq!(run("pub fn f(a: u64) -> i64 { a as i64 }").len(), 1);
    }

    #[test]
    fn widening_cast_is_not_flagged() {
        assert!(run("pub fn f(a: u32) -> u64 { a as u64 }").is_empty());
    }

    #[test]
    fn checked_and_saturating_methods_are_not_flagged() {
        assert!(run("pub fn f(a: u64, b: u64) -> Option<u64> { a.checked_add(b) }").is_empty());
        assert!(run("pub fn f(a: u64, b: u64) -> u64 { a.saturating_sub(b) }").is_empty());
    }

    #[test]
    fn division_comparison_and_constants_are_not_flagged() {
        assert!(run("pub fn f(a: u64, b: u64) -> bool { a / 2 > b }").is_empty());
        assert!(run("pub fn f() -> u64 { 1 + 2 }").is_empty());
    }

    #[test]
    fn locals_copied_from_a_parameter_are_tracked() {
        let f = run("pub fn f(a: u64) -> u64 { let x = a; x + 1 }");
        assert_eq!(f.len(), 1);
        assert!(
            f[0].message
                .contains("`x` (derived from integer parameter `a`)"),
            "{}",
            f[0].message
        );
    }

    #[test]
    fn locals_derived_through_arithmetic_and_casts_are_tracked() {
        assert_eq!(
            run("pub fn f(a: u32) -> u64 { let x = a as u64; x * 2 }").len(),
            1
        );
        assert_eq!(
            run("pub fn f(a: u64) -> u64 { let x = (a); let y = x; y - 1 }").len(),
            1
        );
        // The cast target's width is used for the derived local.
        let f = run("pub fn f(a: u64) -> i32 { let x = a as u32; x as i32 }");
        assert!(
            f.iter().any(|f| f.message.contains("`as` cast of `x`")),
            "{f:?}"
        );
    }

    #[test]
    fn a_shadowing_binding_that_is_not_tracked_stops_tracking() {
        assert!(run("pub fn f(a: u64) -> u64 { let x = a; let x = 7u64; x + 1 }").is_empty());
    }

    #[test]
    fn locals_not_derived_from_parameters_are_not_tracked() {
        assert!(run("pub fn f(env: Env) -> u64 { let x: u64 = env.storage().instance().get(&K).unwrap(); x + 1 }").is_empty());
        assert!(run("pub fn f(a: u64) -> u64 { let x = 5u64; x + 1 }").is_empty());
    }

    #[test]
    fn a_checked_local_is_not_flagged() {
        assert!(run("pub fn f(a: u64) -> Option<u64> { let x = a; x.checked_add(1) }").is_empty());
    }

    #[test]
    fn follows_a_tracked_argument_into_a_same_file_helper() {
        let f = run_file("fn bump(v: u64) -> u64 { v + 1 } pub struct C; #[contractimpl] impl C { pub fn f(a: u64) -> u64 { bump(a) } }");
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(
            f[0].message
                .starts_with("Helper `bump`, called from public contract function `f`"),
            "{}",
            f[0].message
        );
        assert_eq!(f[0].start_line, Some(1));
    }

    #[test]
    fn follows_through_a_chain_and_a_self_method() {
        let src = "fn inner(v: u64) -> u64 { v * 2 } fn outer(v: u64) -> u64 { inner(v) } pub struct C; #[contractimpl] impl C { pub fn f(a: u64) -> u64 { outer(a) } }";
        let f = run_file(src);
        assert_eq!(f.len(), 1, "{f:?}");
        assert!(f[0].message.contains("Helper `inner`"));
        let g = run_file("pub struct C; impl C { fn h(&self, v: u64) -> u64 { v + 1 } } #[contractimpl] impl C { pub fn f(&self, a: u64) -> u64 { self.h(a) } }");
        assert_eq!(g.len(), 1, "{g:?}");
    }

    #[test]
    fn only_the_positions_that_receive_a_tracked_value_are_tracked() {
        // `limit` comes from a constant, not from the caller's parameter.
        let f = run_file("fn mix(v: u64, limit: u64) -> u64 { limit + 1 } pub struct C; #[contractimpl] impl C { pub fn f(a: u64) -> u64 { mix(a, 10) } }");
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn ambiguous_or_untracked_helpers_are_not_followed() {
        let dup = run_file("fn bump(v: u64) -> u64 { v + 1 } mod m { pub fn bump(v: u64) -> u64 { v + 2 } } pub struct C; #[contractimpl] impl C { pub fn f(a: u64) -> u64 { bump(a) } }");
        assert!(dup.is_empty(), "{dup:?}");
        let untracked = run_file("fn bump(v: u64) -> u64 { v + 1 } pub struct C; #[contractimpl] impl C { pub fn f(a: u64) -> u64 { bump(3) } }");
        assert!(untracked.is_empty(), "{untracked:?}");
    }

    #[test]
    fn recursive_helpers_terminate() {
        let f = run_file("fn a(v: u64) -> u64 { b(v) + 1 } fn b(v: u64) -> u64 { a(v) } pub struct C; #[contractimpl] impl C { pub fn f(x: u64) -> u64 { a(x) } }");
        assert!(f.len() <= 2, "{f:?}");
    }

    #[test]
    fn an_entrypoint_called_as_a_helper_is_reported_once() {
        let f = run_file("pub struct C; #[contractimpl] impl C { pub fn g(a: u64) -> u64 { a + 1 } pub fn f(a: u64) -> u64 { Self::g(a) } }");
        assert_eq!(f.len(), 1, "{f:?}");
    }

    #[test]
    fn non_integer_parameters_are_ignored() {
        assert!(run("pub fn f(a: Address, b: Symbol) { let _ = a + b; }").is_empty());
    }

    #[test]
    fn private_and_non_contract_functions_are_ignored() {
        assert!(run("fn f(a: u64, b: u64) -> u64 { a + b }").is_empty());
        let ast =
            syn::parse_file("pub struct D; impl D { pub fn f(a: u64) -> u64 { a + 1 } }").unwrap();
        assert!(ArithmeticBoundaryReview
            .check(&SourceUnit {
                path: "x.rs",
                ast: &ast
            })
            .is_empty());
    }

    #[test]
    fn one_finding_per_function() {
        let f = run("pub fn f(a: u64, b: u64) -> u64 { a + b + a * b }");
        assert_eq!(f.len(), 1);
        assert!(f[0].message.contains("2 arithmetic"));
    }

    #[test]
    fn skips_cfg_test_modules() {
        let ast = syn::parse_file("#[cfg(test)] mod t { pub struct C; #[contractimpl] impl C { pub fn f(a: u64) -> u64 { a + 1 } } }").unwrap();
        assert!(ArithmeticBoundaryReview
            .check(&SourceUnit {
                path: "x.rs",
                ast: &ast
            })
            .is_empty());
    }
}
