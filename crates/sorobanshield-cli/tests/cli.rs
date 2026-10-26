use std::path::PathBuf;
use std::process::{Command, Output};

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sorobanshield"))
        .args(args)
        .output()
        .unwrap()
}

fn scan(rel: &str, extra: &[&str]) -> Output {
    let p = fixture(rel);
    let mut args = vec!["scan", p.to_str().unwrap()];
    args.extend_from_slice(extra);
    run(&args)
}

fn code(o: &Output) -> i32 {
    o.status.code().unwrap()
}

fn out(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn help_and_version() {
    let h = run(&["--help"]);
    assert_eq!(code(&h), 0);
    assert!(
        out(&h).contains("scan") && out(&h).contains("rules") && out(&h).contains("Exit codes")
    );
    let v = run(&["--version"]);
    assert_eq!(code(&v), 0);
    assert!(out(&v).starts_with("sorobanshield 0.1.0"));
}

#[test]
fn positive_fixture_reports_finding_and_exits_1() {
    let o = scan("positive/unprotected_write", &[]);
    assert_eq!(code(&o), 1);
    let t = out(&o);
    assert!(
        t.contains("SS-001")
            && t.contains("src/lib.rs:10")
            && t.contains("severity: medium, confidence: low"),
        "{t}"
    );
}

#[test]
fn negative_fixtures_are_clean() {
    for f in [
        "negative/protected_write",
        "negative/helper_auth",
        "edge/test_module_ignored",
    ] {
        let o = scan(f, &[]);
        assert_eq!(code(&o), 0, "{f}: {}", out(&o));
        assert!(out(&o).contains("No findings reported"));
    }
}

#[test]
fn json_output_is_valid_and_stable() {
    let o = scan("positive/unprotected_write", &["--format", "json"]);
    assert_eq!(code(&o), 1);
    let v: serde_json::Value = serde_json::from_str(&out(&o)).expect("stdout is pure JSON");
    assert_eq!(v["schema_version"], 1);
    assert_eq!(v["summary"]["total"], 1);
    assert_eq!(v["findings"][0]["rule_id"], "SS-001");
    assert_eq!(v["findings"][0]["file"], "src/lib.rs");
    assert_eq!(v["findings"][0]["start_line"], 10);
    assert_eq!(v["scan"]["files_scanned"], 1);
    assert_eq!(
        out(&o),
        out(&scan("positive/unprotected_write", &["--format", "json"]))
    );
}

#[test]
fn fail_on_threshold_controls_exit_code() {
    assert_eq!(
        code(&scan("positive/unprotected_write", &["--fail-on", "high"])),
        0
    );
    assert_eq!(
        code(&scan(
            "positive/unprotected_write",
            &["--fail-on", "medium"]
        )),
        1
    );
}

#[test]
fn invalid_fail_on_is_usage_error() {
    assert_eq!(
        code(&scan(
            "positive/unprotected_write",
            &["--fail-on", "urgent"]
        )),
        2
    );
}

#[test]
fn operational_failures_exit_3_and_never_look_clean() {
    let missing = run(&["scan", "/definitely/not/here"]);
    assert_eq!(code(&missing), 3);
    assert!(err(&missing).contains("does not exist"));

    let manifest = scan("invalid/bad_manifest", &[]);
    assert_eq!(code(&manifest), 3);
    assert!(err(&manifest).contains("malformed manifest"));

    let not_soroban = scan("negative/not_soroban", &[]);
    assert_eq!(code(&not_soroban), 3);
    assert!(err(&not_soroban).contains("soroban-sdk"));

    let file = fixture("positive/unprotected_write/src/lib.rs");
    assert_eq!(code(&run(&["scan", file.to_str().unwrap()])), 3);
}

#[test]
fn unparseable_source_makes_scan_incomplete() {
    let o = scan("invalid/malformed_source", &[]);
    assert_eq!(code(&o), 3);
    assert!(out(&o).contains("NOT a clean result"));
    assert!(!out(&o).contains("No findings reported"));
    let j = scan("invalid/malformed_source", &["--format", "json"]);
    let v: serde_json::Value = serde_json::from_str(&out(&j)).unwrap();
    assert_eq!(v["scan"]["errors"].as_array().unwrap().len(), 1);
}

#[test]
fn rules_list_marks_planned_rules() {
    let o = run(&["rules", "list"]);
    assert_eq!(code(&o), 0);
    let t = out(&o);
    assert!(t.contains("SS-001") && t.contains("[implemented]"));
    assert!(t.contains("SS-002") && t.contains("[planned]"));
    let j: serde_json::Value =
        serde_json::from_str(&out(&run(&["rules", "list", "--format", "json"]))).unwrap();
    assert_eq!(j.as_array().unwrap().len(), 6);
}

#[test]
fn rules_explain_known_and_unknown() {
    let o = run(&["rules", "explain", "SS-001"]);
    assert_eq!(code(&o), 0);
    assert!(out(&o).contains("Known false negatives"));
    let bad = run(&["rules", "explain", "SS-999"]);
    assert_eq!(code(&bad), 2);
    assert!(err(&bad).contains("unknown rule"));
}

#[test]
fn unknown_command_is_usage_error() {
    assert_eq!(code(&run(&["frobnicate"])), 2);
}
