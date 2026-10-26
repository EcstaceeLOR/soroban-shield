use syn::{Attribute, Expr, ImplItem, ImplItemFn, Item, ItemImpl, Visibility};

/// Method names that write contract storage when called on a `.storage()` chain.
pub const WRITE_CALLS: &[&str] = &["set", "remove"];

pub fn has_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|a| a.path().is_ident(name))
}

/// True for `#[cfg(test)]` and `#[cfg(any(test, ..))]`-style attributes.
pub fn is_cfg_test(attrs: &[Attribute]) -> bool {
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

/// Whether the receiver chain of a method call contains a `.storage()` call.
pub fn receiver_chain_has_storage(mut e: &Expr) -> bool {
    loop {
        match e {
            Expr::MethodCall(m) => {
                if m.method == "storage" {
                    return true;
                }
                e = &m.receiver;
            }
            Expr::Paren(p) => e = &p.expr,
            Expr::Reference(r) => e = &r.expr,
            _ => return false,
        }
    }
}

/// Public functions of `#[contractimpl]` impl blocks, skipping `#[cfg(test)]` items.
pub fn contract_entrypoints(items: &[Item]) -> Vec<&ImplItemFn> {
    fn walk<'a>(items: &'a [Item], out: &mut Vec<&'a ImplItemFn>) {
        for item in items {
            match item {
                Item::Impl(ItemImpl { attrs, items, .. })
                    if !is_cfg_test(attrs) && has_attr(attrs, "contractimpl") =>
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
                        walk(inner, out);
                    }
                }
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    walk(items, &mut out);
    out
}

/// Spans of storage writes (`.set` / `.remove` on a `.storage()` chain) inside a block.
pub fn storage_write_spans(block: &syn::Block) -> Vec<proc_macro2::Span> {
    use syn::visit::Visit;
    struct V(Vec<proc_macro2::Span>);
    impl<'ast> Visit<'ast> for V {
        fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
            if WRITE_CALLS.contains(&m.method.to_string().as_str())
                && receiver_chain_has_storage(&m.receiver)
            {
                self.0.push(m.method.span());
            }
            syn::visit::visit_expr_method_call(self, m);
        }
    }
    let mut v = V(Vec::new());
    v.visit_block(block);
    v.0
}
