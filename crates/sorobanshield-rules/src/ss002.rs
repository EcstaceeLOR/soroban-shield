use proc_macro2::Span;
use sorobanshield_core::{Confidence, Finding, Rule, RuleMeta, RuleStatus, Severity, SourceUnit};
use std::collections::HashMap;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    BinOp, Expr, ExprBinary, ExprCast, FnArg, ImplItem, Item, ItemImpl, Pat, Type, Visibility,
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

fn param_ident(e: &Expr) -> Option<String> {
    match e {
        Expr::Path(p) if p.path.segments.len() == 1 => Some(p.path.segments[0].ident.to_string()),
        Expr::Paren(p) => param_ident(&p.expr),
        _ => None,
    }
}

struct Hits<'a> {
    params: &'a HashMap<String, (u32, bool)>,
    found: Vec<(Span, String)>,
}

impl<'ast> Visit<'ast> for Hits<'_> {
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
                .filter_map(|s| param_ident(s))
                .find(|n| self.params.contains_key(n));
            if let Some(name) = name {
                self.found.push((
                    b.op.span(),
                    format!("unchecked `{op}` on integer parameter `{name}`"),
                ));
            }
        }
        syn::visit::visit_expr_binary(self, b);
    }

    fn visit_expr_cast(&mut self, c: &'ast ExprCast) {
        if let (Some(name), Some((to_bits, to_signed))) = (param_ident(&c.expr), type_kind(&c.ty)) {
            if let Some(&(from_bits, from_signed)) = self.params.get(&name) {
                if to_bits < from_bits || (to_bits == from_bits && to_signed != from_signed) {
                    self.found.push((
                        c.as_token.span,
                        format!(
                            "`as` cast of integer parameter `{name}` may truncate or change sign"
                        ),
                    ));
                }
            }
        }
        syn::visit::visit_expr_cast(self, c);
    }
}

fn is_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| match &a.meta {
        syn::Meta::List(l) => {
            a.path().is_ident("cfg")
                && l.tokens
                    .to_string()
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .any(|t| t == "test")
        }
        _ => false,
    })
}

fn collect<'a>(items: &'a [Item], out: &mut Vec<&'a syn::ImplItemFn>) {
    for item in items {
        match item {
            Item::Impl(ItemImpl { attrs, items, .. })
                if !is_cfg_test(attrs)
                    && attrs.iter().any(|a| a.path().is_ident("contractimpl")) =>
            {
                for it in items {
                    if let ImplItem::Fn(f) = it {
                        if matches!(f.vis, Visibility::Public(_)) {
                            out.push(f);
                        }
                    }
                }
            }
            Item::Mod(m) if !is_cfg_test(&m.attrs) => {
                if let Some((_, inner)) = &m.content {
                    collect(inner, out);
                }
            }
            _ => {}
        }
    }
}

impl Rule for ArithmeticBoundaryReview {
    fn meta(&self) -> RuleMeta {
        RuleMeta {
            id: ID.into(),
            name: "Arithmetic boundary review".into(),
            status: RuleStatus::Implemented,
            severity: Some(Severity::Low),
            confidence: Some(Confidence::Low),
            concern: "Public contract functions that apply +, - or * (or their assignment forms) directly to an integer argument, or cast an integer argument to a narrower or differently signed type, may overflow, underflow, truncate or change sign on attacker-chosen input. Whether that is a problem depends on the build's overflow-checks setting and on validation this rule cannot see.".into(),
            detection: "Parses each file with syn. In public functions of #[contractimpl] impl blocks it records parameters whose declared type is one of i32, u32, i64, u64, i128 or u128. It reports a function when the body applies +, -, *, +=, -= or *= with such a parameter as a direct operand, or uses `param as T` where T is narrower than the parameter type or has the same width but different signedness. Calls such as checked_add, saturating_sub or try_into are method calls and are never reported. #[cfg(test)] items are skipped.".into(),
            severity_rationale: "Low severity: with overflow-checks enabled (as Soroban contract release profiles commonly do), overflow traps instead of wrapping, so the common outcome is a failed invocation rather than corrupted state; truncating casts and builds without overflow checks are worse. Low confidence: only direct uses of parameters are visible, and domain constraints that bound the values are not understood. This is a review prompt, not a vulnerability claim.".into(),
            remediation: "Decide what should happen at the boundary. Use checked_add / checked_sub / checked_mul and return a contract error, or saturating/wrapping variants if that is intended. Replace narrowing `as` casts with try_into and handle the error. Validate argument ranges, confirm overflow-checks = true in the release profile, and add tests at the numeric boundaries.".into(),
            false_positives: vec![
                "Arithmetic on values already bounded by earlier validation or by the domain (for example a small loop counter parameter).".into(),
                "Builds with overflow-checks enabled, where overflow traps deterministically and that is acceptable.".into(),
                "Intentional wrapping or truncating behavior.".into(),
            ],
            false_negatives: vec![
                "Arithmetic on local variables, struct fields, storage reads or function results: only direct parameter operands are tracked.".into(),
                "Operations inside private helper functions, other files, or macros.".into(),
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
                "tests/fixtures/edge/local_arithmetic_untracked".into(),
            ],
        }
    }

    fn check(&self, unit: &SourceUnit<'_>) -> Vec<Finding> {
        let meta = self.meta();
        let mut fns = Vec::new();
        collect(&unit.ast.items, &mut fns);

        let mut out = Vec::new();
        for f in fns {
            let params: HashMap<String, (u32, bool)> = f
                .sig
                .inputs
                .iter()
                .filter_map(|a| match a {
                    FnArg::Typed(t) => match (&*t.pat, type_kind(&t.ty)) {
                        (Pat::Ident(i), Some(k)) => Some((i.ident.to_string(), k)),
                        _ => None,
                    },
                    _ => None,
                })
                .collect();
            if params.is_empty() {
                continue;
            }
            let mut hits = Hits {
                params: &params,
                found: Vec::new(),
            };
            hits.visit_block(&f.block);
            if hits.found.is_empty() {
                continue;
            }
            let first = hits
                .found
                .iter()
                .map(|(s, _)| s.start().line)
                .min()
                .unwrap_or(0);
            let last = hits
                .found
                .iter()
                .map(|(s, _)| s.end().line)
                .max()
                .unwrap_or(first);
            let detail = hits
                .found
                .iter()
                .map(|(_, d)| d.as_str())
                .take(3)
                .collect::<Vec<_>>()
                .join("; ");
            let more = hits.found.len().saturating_sub(3);
            out.push(Finding {
                rule_id: ID.into(),
                title: meta.name.clone(),
                severity: Severity::Low,
                confidence: Confidence::Low,
                message: format!(
                    "Public contract function `{}` has {} arithmetic or cast site(s) on integer arguments that deserve a boundary review: {}{}. Whether this can overflow depends on overflow-checks and on validation not visible to this rule.",
                    f.sig.ident,
                    hits.found.len(),
                    detail,
                    if more > 0 { format!(" (and {more} more)") } else { String::new() }
                ),
                file: unit.path.to_string(),
                start_line: Some(first),
                end_line: Some(last),
                remediation: meta.remediation.clone(),
                references: meta.references.clone(),
                limitations: vec![
                    "Only direct uses of integer parameters are tracked; locals, fields and storage values are not.".into(),
                    "Overflow-checks settings and surrounding validation are not evaluated.".into(),
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
    fn locals_are_not_tracked() {
        assert!(run("pub fn f(a: u64) -> u64 { let x = a; x + 1 }").is_empty());
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
