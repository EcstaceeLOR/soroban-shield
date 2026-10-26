use clap::{Parser, Subcommand, ValueEnum};
use sorobanshield_core::{discover_and_scan, Baseline, Config, Report, RuleMeta, Severity};
use std::path::PathBuf;
use std::process::ExitCode;

const EXIT_OK: u8 = 0;
const EXIT_FINDINGS: u8 = 1;
const EXIT_USAGE: u8 = 2;
const EXIT_FAILURE: u8 = 3;

#[derive(Parser)]
#[command(
    name = "sorobanshield",
    version,
    about = "Security review toolkit for Soroban smart contracts",
    after_help = "Exit codes: 0 no findings at or above --fail-on; 1 findings at or above --fail-on; 2 invalid usage or configuration; 3 scan could not be completed.\nStatic analysis cannot prove a contract is secure."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Text,
    Json,
}

#[derive(Clone, Copy, ValueEnum)]
enum ScanFormat {
    Text,
    Json,
    /// SARIF 2.1.0, for code-scanning tools
    Sarif,
}

#[derive(Subcommand)]
enum Command {
    /// Scan a Soroban project directory
    Scan {
        /// Project root containing Cargo.toml
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, value_enum, default_value = "text")]
        format: ScanFormat,
        /// Exit with code 1 when a finding has at least this severity (info, low, medium, high, critical).
        /// Overrides the config file; defaults to low.
        #[arg(long, value_parser = parse_severity)]
        fail_on: Option<Severity>,
        /// Configuration file. Defaults to <PATH>/sorobanshield.toml when that file exists
        #[arg(long)]
        config: Option<PathBuf>,
        /// Hide findings listed in this baseline file (created with --write-baseline)
        #[arg(long, conflicts_with = "write_baseline")]
        baseline: Option<PathBuf>,
        /// Write the current findings to this baseline file and exit 0 if the scan completed
        #[arg(long)]
        write_baseline: Option<PathBuf>,
    },
    /// Inspect the rule catalog
    Rules {
        #[command(subcommand)]
        action: RulesCommand,
    },
}

#[derive(Subcommand)]
enum RulesCommand {
    /// List all rules, including planned ones (planned rules are never run)
    List {
        #[arg(long, value_enum, default_value = "text")]
        format: Format,
    },
    /// Explain one rule, including its limitations
    Explain {
        rule_id: String,
        #[arg(long, value_enum, default_value = "text")]
        format: Format,
    },
}

fn parse_severity(s: &str) -> Result<Severity, String> {
    s.parse()
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    ExitCode::from(match cli.command {
        Command::Scan {
            path,
            format,
            fail_on,
            config,
            baseline,
            write_baseline,
        } => scan(ScanArgs {
            path,
            format,
            fail_on,
            config,
            baseline,
            write_baseline,
        }),
        Command::Rules {
            action: RulesCommand::List { format },
        } => list(format),
        Command::Rules {
            action: RulesCommand::Explain { rule_id, format },
        } => explain(&rule_id, format),
    })
}

struct ScanArgs {
    path: PathBuf,
    format: ScanFormat,
    fail_on: Option<Severity>,
    config: Option<PathBuf>,
    baseline: Option<PathBuf>,
    write_baseline: Option<PathBuf>,
}

fn scan(args: ScanArgs) -> u8 {
    let ScanArgs {
        path,
        format,
        fail_on,
        config,
        baseline,
        write_baseline,
    } = args;
    let runnable: Vec<String> = sorobanshield_rules::active_rules()
        .iter()
        .map(|r| r.meta().id)
        .collect();

    let (mut cfg, config_shown) = match load_config(&path, config) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            return EXIT_USAGE;
        }
    };
    if let Err(e) = cfg.validate_rules(&runnable) {
        eprintln!("error: {e}");
        return EXIT_USAGE;
    }
    let loaded_baseline = match baseline.as_deref().map(Baseline::load).transpose() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: {e}");
            return EXIT_USAGE;
        }
    };

    let rules: Vec<_> = sorobanshield_rules::active_rules()
        .into_iter()
        .filter(|r| !cfg.rules.disable.contains(&r.meta().id))
        .collect();
    let (_project, mut outcome) = match discover_and_scan(&path, &cfg.scan.exclude, &rules) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            return EXIT_FAILURE;
        }
    };

    let (mut suppressed, mut unmatched) = (0, 0);
    if let Some(b) = &loaded_baseline {
        let applied = b.apply(std::mem::take(&mut outcome.findings));
        outcome.findings = applied.kept;
        suppressed = applied.suppressed;
        unmatched = applied.unmatched;
    }

    let ids = rules.iter().map(|r| r.meta().id).collect();
    let mut report = Report::new(
        env!("CARGO_PKG_VERSION"),
        &path.display().to_string(),
        ids,
        outcome,
    );
    report.scan.config_file = config_shown;
    report.scan.exclude = cfg.scan.exclude.clone();
    report.scan.disabled_rules = cfg.rules.disable.clone();
    report.scan.baseline_file = baseline.as_ref().map(|b| b.display().to_string());
    report.scan.baseline_unmatched = unmatched;
    report.summary.suppressed_by_baseline = suppressed;

    match format {
        ScanFormat::Json => println!("{}", report.to_json()),
        ScanFormat::Text => print!("{}", report.to_terminal()),
        ScanFormat::Sarif => {
            let metas: Vec<_> = rules.iter().map(|r| r.meta()).collect();
            println!("{}", report.to_sarif(&metas));
        }
    }
    if report.has_errors() {
        eprintln!(
            "error: {} file(s) could not be analyzed; the scan is incomplete{}",
            report.scan.errors.len(),
            if write_baseline.is_some() {
                ", so no baseline was written"
            } else {
                ""
            }
        );
        return EXIT_FAILURE;
    }
    if let Some(target) = write_baseline {
        let b = Baseline::from_findings(&report.findings);
        return match b.save(&target) {
            Ok(()) => {
                eprintln!(
                    "wrote baseline with {} entr(ies) to {}",
                    b.fingerprints.len(),
                    target.display()
                );
                EXIT_OK
            }
            Err(e) => {
                eprintln!("error: {e}");
                EXIT_FAILURE
            }
        };
    }
    let threshold = fail_on.or(cfg.scan.fail_on).unwrap_or(Severity::Low);
    if report.exceeds(threshold) {
        EXIT_FINDINGS
    } else {
        EXIT_OK
    }
}

/// Loads the explicit `--config` file, or `<root>/sorobanshield.toml` if present.
fn load_config(
    root: &std::path::Path,
    explicit: Option<PathBuf>,
) -> Result<(Config, Option<String>), String> {
    let (file, shown) = match explicit {
        Some(f) => {
            if !f.is_file() {
                return Err(format!("config file not found: {}", f.display()));
            }
            let shown = f.display().to_string();
            (f, shown)
        }
        None => {
            let f = root.join("sorobanshield.toml");
            if !f.is_file() {
                return Ok((Config::default(), None));
            }
            (f, "sorobanshield.toml".to_string())
        }
    };
    Config::load(&file)
        .map(|c| (c, Some(shown)))
        .map_err(|e| e.to_string())
}

fn list(format: Format) -> u8 {
    let catalog = sorobanshield_rules::catalog();
    match format {
        Format::Json => println!(
            "{}",
            serde_json::to_string_pretty(&catalog).expect("catalog serialises")
        ),
        Format::Text => {
            for m in &catalog {
                println!("{}  {:<13} {}", m.id, format!("[{}]", status(m)), m.name);
            }
            println!("\nOnly rules marked [implemented] run during a scan. Use `sorobanshield rules explain <ID>` for details.");
        }
    }
    EXIT_OK
}

fn status(m: &RuleMeta) -> String {
    serde_json::to_value(m.status)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}

fn explain(id: &str, format: Format) -> u8 {
    let Some(m) = sorobanshield_rules::find_rule(id) else {
        eprintln!(
            "error: unknown rule '{id}'. Run `sorobanshield rules list` to see available rules."
        );
        return EXIT_USAGE;
    };
    match format {
        Format::Json => println!(
            "{}",
            serde_json::to_string_pretty(&m).expect("rule serialises")
        ),
        Format::Text => {
            println!("{} - {} [{}]", m.id, m.name, status(&m));
            if let (Some(s), Some(c)) = (m.severity, m.confidence) {
                println!("Severity: {s}   Confidence: {c}");
            }
            println!("\nConcern:\n  {}\n\nDetection:\n  {}\n\nSeverity rationale:\n  {}\n\nRemediation:\n  {}", m.concern, m.detection, m.severity_rationale, m.remediation);
            for (title, items) in [
                ("Known false positives", &m.false_positives),
                ("Known false negatives", &m.false_negatives),
                ("References", &m.references),
                ("Fixtures", &m.fixtures),
            ] {
                if !items.is_empty() {
                    println!("\n{title}:");
                    for i in items {
                        println!("  - {i}");
                    }
                }
            }
        }
    }
    EXIT_OK
}
