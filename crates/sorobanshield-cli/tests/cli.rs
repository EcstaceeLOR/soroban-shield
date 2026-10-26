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
    assert!(t.contains("SS-002") && t.contains("SS-003") && t.contains("[planned]"));
    assert!(t
        .lines()
        .any(|l| l.starts_with("SS-002") && l.contains("[implemented]")));
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

#[test]
fn ss002_flags_unchecked_arithmetic_and_casts() {
    let o = scan("positive/unchecked_arithmetic", &["--format", "json"]);
    assert_eq!(code(&o), 1);
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    let f = v["findings"].as_array().unwrap();
    assert_eq!(f.len(), 2);
    assert!(f
        .iter()
        .all(|x| x["rule_id"] == "SS-002" && x["severity"] == "low" && x["confidence"] == "low"));
    assert_eq!(f[0]["start_line"], 10);
    assert_eq!(f[1]["start_line"], 14);
    assert_eq!(
        v["scan"]["rules_run"],
        serde_json::json!(["SS-001", "SS-002", "SS-006"])
    );
}

#[test]
fn ss002_negative_and_edge_fixtures_are_clean() {
    for f in [
        "negative/checked_arithmetic",
        "edge/local_arithmetic_untracked",
    ] {
        assert_eq!(code(&scan(f, &[])), 0, "{f}");
    }
}

#[test]
fn ss002_low_severity_respects_fail_on() {
    assert_eq!(
        code(&scan(
            "positive/unchecked_arithmetic",
            &["--fail-on", "medium"]
        )),
        0
    );
}

#[test]
fn ss006_reports_five_informational_diagnostics() {
    let o = scan("positive/poor_metadata", &["--format", "json"]);
    assert_eq!(
        code(&o),
        0,
        "info findings are below the default --fail-on low"
    );
    let v: serde_json::Value = serde_json::from_str(&out(&o)).unwrap();
    let f = v["findings"].as_array().unwrap();
    assert_eq!(f.len(), 5);
    assert!(f
        .iter()
        .all(|x| x["rule_id"] == "SS-006" && x["severity"] == "info" && x["file"] == "Cargo.toml"));
    assert!(f.iter().all(|x| x["start_line"].is_null()));
    assert_eq!(v["summary"]["by_severity"]["info"], 5);
    let text = f
        .iter()
        .map(|x| x["message"].as_str().unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    for needle in [
        "wildcard",
        "cdylib",
        "overflow-checks",
        "Cargo.lock",
        "rust-toolchain",
    ] {
        assert!(text.contains(needle), "missing diagnostic about {needle}");
    }
}

#[test]
fn ss006_fails_the_build_only_when_asked() {
    assert_eq!(
        code(&scan("positive/poor_metadata", &["--fail-on", "info"])),
        1
    );
    assert_eq!(code(&scan("positive/poor_metadata", &[])), 0);
}

#[test]
fn ss006_terminal_output_has_no_line_suffix() {
    let t = out(&scan("positive/poor_metadata", &[]));
    assert!(t.contains("at Cargo.toml\n"), "{t}");
    assert!(t.contains("severity: info"));
}

fn json(o: &Output) -> serde_json::Value {
    serde_json::from_str(&out(o)).expect("stdout is pure JSON")
}

#[test]
fn config_can_disable_a_rule_and_report_says_so() {
    let o = scan("config/disable_rule", &["--format", "json"]);
    assert_eq!(code(&o), 0);
    let v = json(&o);
    assert_eq!(
        v["scan"]["rules_run"],
        serde_json::json!(["SS-002", "SS-006"])
    );
    assert_eq!(v["scan"]["disabled_rules"], serde_json::json!(["SS-001"]));
    assert_eq!(v["scan"]["config_file"], "sorobanshield.toml");
    assert_eq!(v["summary"]["total"], 0);
}

#[test]
fn config_exclude_skips_paths_and_is_visible() {
    let o = scan("config/exclude_path", &["--format", "json"]);
    assert_eq!(code(&o), 0);
    let v = json(&o);
    assert_eq!(v["scan"]["files_scanned"], 1);
    assert_eq!(v["scan"]["exclude"], serde_json::json!(["generated"]));

    let empty = fixture("config/files/empty.toml");
    let o = scan(
        "config/exclude_path",
        &["--config", empty.to_str().unwrap(), "--format", "json"],
    );
    assert_eq!(
        code(&o),
        1,
        "without the exclusion the generated file is scanned"
    );
    let v = json(&o);
    assert_eq!(v["scan"]["files_scanned"], 2);
    assert_eq!(v["findings"][0]["file"], "generated/bad.rs");
}

#[test]
fn config_fail_on_applies_and_cli_flag_overrides_it() {
    assert_eq!(code(&scan("config/fail_on_high", &[])), 0);
    assert_eq!(code(&scan("config/fail_on_high", &["--fail-on", "low"])), 1);
}

#[test]
fn terminal_output_mentions_the_applied_config() {
    let t = out(&scan("config/exclude_path", &[]));
    assert!(
        t.contains("config: sorobanshield.toml (1 excluded path(s), 0 rule(s) disabled)"),
        "{t}"
    );
}

#[test]
fn invalid_configuration_is_a_usage_error_and_never_a_clean_scan() {
    let cases = [
        ("unknown_key.toml", "unknown field"),
        ("bad_severity.toml", "invalid config file"),
        ("unknown_rule.toml", "SS-999"),
        ("disable_all.toml", "every rule"),
        ("absolute_exclude.toml", "relative path"),
        ("parent_exclude.toml", ".."),
        ("glob_exclude.toml", "globs are not supported"),
        ("syntax_error.toml", "invalid config file"),
    ];
    for (file, needle) in cases {
        let cfg = fixture(&format!("config/files/{file}"));
        let o = scan(
            "negative/good_metadata",
            &["--config", cfg.to_str().unwrap()],
        );
        assert_eq!(code(&o), 2, "{file}: {}", err(&o));
        assert!(err(&o).contains(needle), "{file}: {}", err(&o));
        assert!(out(&o).is_empty(), "{file} must not print a report");
    }
}

#[test]
fn missing_explicit_config_is_a_usage_error() {
    let o = scan(
        "negative/good_metadata",
        &["--config", "/no/such/sorobanshield.toml"],
    );
    assert_eq!(code(&o), 2);
    assert!(err(&o).contains("not found"));
}

#[test]
fn malformed_auto_discovered_config_is_not_ignored() {
    let d = std::env::temp_dir().join(format!("ss-cfg-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("src")).unwrap();
    std::fs::write(
        d.join("Cargo.toml"),
        "[package]\nname=\"x\"\nversion=\"0.1.0\"\n[dependencies]\nsoroban-sdk=\"22\"\n",
    )
    .unwrap();
    std::fs::write(d.join("src/lib.rs"), "").unwrap();
    std::fs::write(d.join("sorobanshield.toml"), "[scan\n").unwrap();
    let o = run(&["scan", d.to_str().unwrap()]);
    let _ = std::fs::remove_dir_all(&d);
    assert_eq!(code(&o), 2);
}

#[test]
fn sarif_output_has_expected_structure() {
    let o = scan("positive/unprotected_write", &["--format", "sarif"]);
    assert_eq!(code(&o), 1);
    let v = json(&o);
    assert_eq!(v["version"], "2.1.0");
    let run = &v["runs"][0];
    assert_eq!(run["tool"]["driver"]["name"], "SorobanShield");
    let rules = run["tool"]["driver"]["rules"].as_array().unwrap();
    assert_eq!(rules.len(), 3);
    assert_eq!(rules[0]["id"], "SS-001");
    let r = &run["results"][0];
    assert_eq!(r["ruleId"], "SS-001");
    assert_eq!(r["ruleIndex"], 0);
    assert_eq!(r["level"], "warning");
    let loc = &r["locations"][0]["physicalLocation"];
    assert_eq!(loc["artifactLocation"]["uri"], "src/lib.rs");
    assert_eq!(loc["region"]["startLine"], 10);
    assert_eq!(run["invocations"][0]["executionSuccessful"], true);
    assert_eq!(
        out(&o),
        out(&scan("positive/unprotected_write", &["--format", "sarif"]))
    );
}

#[test]
fn sarif_levels_and_manifest_findings_without_regions() {
    let v = json(&scan("positive/poor_metadata", &["--format", "sarif"]));
    let results = v["runs"][0]["results"].as_array().unwrap();
    assert_eq!(results.len(), 5);
    for r in results {
        assert_eq!(r["level"], "note");
        let phys = &r["locations"][0]["physicalLocation"];
        assert_eq!(phys["artifactLocation"]["uri"], "Cargo.toml");
        assert!(
            phys.get("region").is_none(),
            "manifest findings have no line numbers"
        );
    }
    let low = json(&scan(
        "positive/unchecked_arithmetic",
        &["--format", "sarif"],
    ));
    assert_eq!(low["runs"][0]["results"][0]["level"], "note");
}

#[test]
fn sarif_reports_failed_scans_as_unsuccessful() {
    let o = scan("invalid/malformed_source", &["--format", "sarif"]);
    assert_eq!(code(&o), 3);
    let inv = &json(&o)["runs"][0]["invocations"][0];
    assert_eq!(inv["executionSuccessful"], false);
    assert_eq!(
        inv["toolExecutionNotifications"].as_array().unwrap().len(),
        1
    );
}

#[test]
fn rules_commands_do_not_accept_sarif() {
    assert_eq!(code(&run(&["rules", "list", "--format", "sarif"])), 2);
}

fn temp_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ss-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn copy_dir(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let dest = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &dest);
        } else {
            std::fs::copy(e.path(), dest).unwrap();
        }
    }
}

#[test]
fn baseline_write_then_apply_suppresses_known_findings() {
    let d = temp_dir("baseline-basic");
    let base = d.join("baseline.json");
    let w = scan(
        "positive/unprotected_write",
        &["--write-baseline", base.to_str().unwrap()],
    );
    assert_eq!(
        code(&w),
        0,
        "writing a baseline succeeds when the scan completes"
    );
    assert!(err(&w).contains("wrote baseline with 1 entr"));
    let b: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&base).unwrap()).unwrap();
    assert_eq!(b["schema_version"], 1);
    assert_eq!(b["fingerprints"].as_array().unwrap().len(), 1);

    let o = scan(
        "positive/unprotected_write",
        &["--baseline", base.to_str().unwrap(), "--format", "json"],
    );
    assert_eq!(code(&o), 0);
    let v = json(&o);
    assert_eq!(v["summary"]["total"], 0);
    assert_eq!(v["summary"]["suppressed_by_baseline"], 1);
    assert_eq!(v["scan"]["baseline_unmatched"], 0);
    assert!(v["scan"]["baseline_file"]
        .as_str()
        .unwrap()
        .ends_with("baseline.json"));
    let t = out(&scan(
        "positive/unprotected_write",
        &["--baseline", base.to_str().unwrap()],
    ));
    assert!(t.contains("1 finding(s) suppressed, 0 stale"), "{t}");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn baseline_survives_line_shifts_but_not_new_findings() {
    let d = temp_dir("baseline-shift");
    let proj = d.join("proj");
    copy_dir(&fixture("positive/unprotected_write"), &proj);
    let base = d.join("baseline.json");
    let run_scan = |extra: &[&str]| {
        let mut a = vec!["scan", proj.to_str().unwrap(), "--format", "json"];
        a.extend_from_slice(extra);
        run(&a)
    };
    assert_eq!(
        code(&run_scan(&["--write-baseline", base.to_str().unwrap()])),
        0
    );

    let lib = proj.join("src/lib.rs");
    let original = std::fs::read_to_string(&lib).unwrap();
    std::fs::write(&lib, format!("\n\n\n// moved down\n{original}")).unwrap();
    let moved = run_scan(&["--baseline", base.to_str().unwrap()]);
    assert_eq!(
        code(&moved),
        0,
        "same finding on a new line is still suppressed"
    );
    assert_eq!(json(&moved)["summary"]["suppressed_by_baseline"], 1);

    std::fs::write(
        &lib,
        format!("{original}\n#[contractimpl]\nimpl Counter {{\n    pub fn reset(env: Env) {{\n        env.storage().instance().remove(&symbol_short!(\"VALUE\"));\n    }}\n}}\n"),
    )
    .unwrap();
    let fresh = run_scan(&["--baseline", base.to_str().unwrap()]);
    assert_eq!(code(&fresh), 1, "a new finding is not hidden");
    let v = json(&fresh);
    assert_eq!(v["summary"]["total"], 1);
    assert_eq!(v["summary"]["suppressed_by_baseline"], 1);
    assert!(v["findings"][0]["message"]
        .as_str()
        .unwrap()
        .contains("`reset`"));
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn baseline_reports_stale_entries() {
    let d = temp_dir("baseline-stale");
    let base = d.join("baseline.json");
    assert_eq!(
        code(&scan(
            "positive/unprotected_write",
            &["--write-baseline", base.to_str().unwrap()]
        )),
        0
    );
    let o = scan(
        "negative/good_metadata",
        &["--baseline", base.to_str().unwrap(), "--format", "json"],
    );
    assert_eq!(code(&o), 0);
    assert_eq!(json(&o)["scan"]["baseline_unmatched"], 1);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn baseline_errors_are_usage_errors_and_never_clean() {
    let d = temp_dir("baseline-bad");
    let bad = d.join("bad.json");
    std::fs::write(&bad, "{ nope").unwrap();
    for path in [bad.to_str().unwrap(), "/no/such/baseline.json"] {
        let o = scan("positive/unprotected_write", &["--baseline", path]);
        assert_eq!(code(&o), 2, "{path}");
        assert!(out(&o).is_empty());
    }
    let both = scan(
        "positive/unprotected_write",
        &[
            "--baseline",
            bad.to_str().unwrap(),
            "--write-baseline",
            d.join("x.json").to_str().unwrap(),
        ],
    );
    assert_eq!(code(&both), 2);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn incomplete_scan_does_not_write_a_baseline() {
    let d = temp_dir("baseline-incomplete");
    let base = d.join("baseline.json");
    let o = scan(
        "invalid/malformed_source",
        &["--write-baseline", base.to_str().unwrap()],
    );
    assert_eq!(code(&o), 3);
    assert!(!base.exists());
    assert!(err(&o).contains("no baseline was written"));
    let _ = std::fs::remove_dir_all(&d);
}
