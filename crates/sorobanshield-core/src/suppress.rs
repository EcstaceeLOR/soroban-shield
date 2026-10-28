//! Inline suppression comments.
//!
//! `// sorobanshield-ignore: SS-001, SS-003 -- reason` suppresses findings of
//! the named rules whose start line is the comment's own line or the next line
//! of code (blank lines, other comments and `#[...]` attributes are skipped).
//! When that line opens a `{` block, such as a function, the whole block is
//! covered; the block end is found by counting braces outside string literals,
//! so it is approximate. Suppressions are always reported, and ones that match nothing are flagged.

use crate::finding::Finding;
use serde::Serialize;

const MARKER: &str = "sorobanshield-ignore:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directive {
    pub file: String,
    /// Line of the comment itself, 1-based.
    pub line: usize,
    /// First code line after the comment, 1-based.
    pub target: usize,
    /// Last line covered: the end of the block opened on `target`, else `target`.
    pub end: usize,
    pub rules: Vec<String>,
    pub reason: Option<String>,
}

/// A finding hidden by an inline comment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Suppression {
    pub rule_id: String,
    pub file: String,
    pub line: usize,
    pub reason: Option<String>,
}

/// A rule ID named in a suppression comment that matched no finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UnusedSuppression {
    pub rule_id: String,
    pub file: String,
    pub line: usize,
}

pub fn parse_directives(file: &str, text: &str) -> Vec<Directive> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    for (i, raw) in lines.iter().enumerate() {
        let Some(comment_at) = raw.find("//") else {
            continue;
        };
        let comment = &raw[comment_at..];
        let Some(m) = comment.find(MARKER) else {
            continue;
        };
        let spec = &comment[m + MARKER.len()..];
        let (ids, reason) = match spec.split_once("--") {
            Some((a, b)) => (a, Some(b.trim().to_string()).filter(|r| !r.is_empty())),
            None => (spec, None),
        };
        let rules: Vec<String> = ids
            .split(',')
            .map(|s| s.trim().to_ascii_uppercase())
            .filter(|s| !s.is_empty())
            .collect();
        if rules.is_empty() {
            continue;
        }
        let own_line_comment = raw[..comment_at].trim().is_empty();
        let target = if own_line_comment {
            lines
                .iter()
                .enumerate()
                .skip(i + 1)
                .find(|(_, l)| {
                    let t = l.trim();
                    !(t.is_empty() || t.starts_with("//") || t.starts_with("#["))
                })
                .map(|(j, _)| j + 1)
                .unwrap_or(i + 1)
        } else {
            i + 1
        };
        let end = block_end(&lines, target);
        out.push(Directive {
            file: file.to_string(),
            line: i + 1,
            target,
            end,
            rules,
            reason,
        });
    }
    out
}

/// Last line of the brace block opened on line `target` (1-based), or `target`
/// itself when that line opens no block.
fn block_end(lines: &[&str], target: usize) -> usize {
    let mut depth: i32 = 0;
    let mut opened = false;
    for (offset, line) in lines.iter().skip(target - 1).enumerate() {
        let mut in_str = false;
        let mut prev = ' ';
        for c in line.chars() {
            if in_str {
                if c == '"' && prev != '\\' {
                    in_str = false;
                }
            } else if c == '"' {
                in_str = true;
            } else if c == '/' && prev == '/' {
                break;
            } else if c == '{' {
                depth += 1;
                opened = true;
            } else if c == '}' {
                depth -= 1;
            }
            prev = c;
        }
        if !opened && offset == 0 {
            return target;
        }
        if opened && depth <= 0 {
            return target + offset;
        }
    }
    target
}

/// Splits `findings` into the ones that stay and the ones suppressed, and
/// returns directive entries that matched nothing.
pub fn apply(
    directives: &[Directive],
    findings: Vec<Finding>,
) -> (Vec<Finding>, Vec<Suppression>, Vec<UnusedSuppression>) {
    let mut used = vec![Vec::<bool>::new(); directives.len()];
    for (u, d) in used.iter_mut().zip(directives) {
        *u = vec![false; d.rules.len()];
    }
    let mut kept = Vec::new();
    let mut suppressed = Vec::new();
    'finding: for f in findings {
        if let Some(line) = f.start_line {
            for (di, d) in directives.iter().enumerate() {
                if d.file != f.file || !(line == d.line || (d.target..=d.end).contains(&line)) {
                    continue;
                }
                if let Some(ri) = d
                    .rules
                    .iter()
                    .position(|r| r.eq_ignore_ascii_case(&f.rule_id))
                {
                    used[di][ri] = true;
                    suppressed.push(Suppression {
                        rule_id: f.rule_id.clone(),
                        file: f.file.clone(),
                        line,
                        reason: d.reason.clone(),
                    });
                    continue 'finding;
                }
            }
        }
        kept.push(f);
    }
    let mut unused = Vec::new();
    for (d, flags) in directives.iter().zip(&used) {
        for (r, hit) in d.rules.iter().zip(flags) {
            if !hit {
                unused.push(UnusedSuppression {
                    rule_id: r.clone(),
                    file: d.file.clone(),
                    line: d.line,
                });
            }
        }
    }
    (kept, suppressed, unused)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::{Confidence, Severity};

    fn finding(rule: &str, line: usize) -> Finding {
        Finding {
            rule_id: rule.into(),
            title: "t".into(),
            severity: Severity::Low,
            confidence: Confidence::Low,
            message: "m".into(),
            file: "src/lib.rs".into(),
            start_line: Some(line),
            end_line: Some(line),
            remediation: String::new(),
            references: vec![],
            limitations: vec![],
        }
    }

    #[test]
    fn parses_ids_and_reason() {
        let d = parse_directives(
            "src/lib.rs",
            "// sorobanshield-ignore: ss-001, SS-003 -- audited\nfn a() {}\n",
        );
        assert_eq!(d[0].rules, ["SS-001", "SS-003"]);
        assert_eq!(d[0].reason.as_deref(), Some("audited"));
        assert_eq!(d[0].target, 2);
    }

    #[test]
    fn skips_attributes_and_blank_lines_to_find_target() {
        let d = parse_directives(
            "f",
            "// sorobanshield-ignore: SS-001\n\n#[contractimpl]\n// note\npub fn a() {}\n",
        );
        assert_eq!(d[0].target, 5);
    }

    #[test]
    fn trailing_comment_targets_its_own_line() {
        let d = parse_directives(
            "f",
            "let x = 1; // sorobanshield-ignore: SS-002\nlet y = 2;\n",
        );
        assert_eq!(d[0].target, 1);
    }

    #[test]
    fn ignores_comment_without_rule_ids() {
        assert!(parse_directives("f", "// sorobanshield-ignore:\n").is_empty());
    }

    #[test]
    fn suppresses_only_named_rule_on_target_line() {
        let d = parse_directives(
            "src/lib.rs",
            "// sorobanshield-ignore: SS-001 -- ok\npub fn a() {}\n",
        );
        let (kept, sup, unused) = apply(
            &d,
            vec![
                finding("SS-001", 2),
                finding("SS-002", 2),
                finding("SS-001", 9),
            ],
        );
        assert_eq!(sup.len(), 1);
        assert_eq!(sup[0].reason.as_deref(), Some("ok"));
        assert_eq!(kept.len(), 2);
        assert!(unused.is_empty());
    }

    #[test]
    fn directive_above_a_function_covers_its_body() {
        let src = "// sorobanshield-ignore: SS-001\npub fn a() {\n    write();\n}\npub fn b() {\n    write();\n}\n";
        let d = parse_directives("src/lib.rs", src);
        assert_eq!((d[0].target, d[0].end), (2, 4));
        let (kept, sup, _) = apply(&d, vec![finding("SS-001", 3), finding("SS-001", 6)]);
        assert_eq!(sup.len(), 1);
        assert_eq!(kept[0].start_line, Some(6));
    }

    #[test]
    fn braces_in_strings_and_comments_do_not_extend_the_block() {
        let src = "// sorobanshield-ignore: SS-001\nfn a() { let s = \"}\"; } // {\nfn b() {}\n";
        let d = parse_directives("f", src);
        assert_eq!((d[0].target, d[0].end), (2, 2));
    }

    #[test]
    fn reports_unused_rule_ids() {
        let d = parse_directives(
            "src/lib.rs",
            "// sorobanshield-ignore: SS-001, SS-004\npub fn a() {}\n",
        );
        let (_, sup, unused) = apply(&d, vec![finding("SS-001", 2)]);
        assert_eq!(sup.len(), 1);
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].rule_id, "SS-004");
    }
}
