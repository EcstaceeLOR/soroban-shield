use clap::{Parser, Subcommand, ValueEnum};
use sorobanshield_core::{discover_and_scan, Report, RuleMeta, Severity};
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
    after_help = "Exit codes: 0 no findings at or above --fail-on; 1 findings at or above --fail-on; 2 invalid usage; 3 scan could not be completed.\nStatic analysis cannot prove a contract is secure."
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

#[derive(Subcommand)]
enum Command {
    /// Scan a Soroban project directory
    Scan {
        /// Project root containing Cargo.toml
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(long, value_enum, default_value = "text")]
        format: Format,
        /// Exit with code 1 when a finding has at least this severity (info, low, medium, high, critical)
        #[arg(long, default_value = "low", value_parser = parse_severity)]
        fail_on: Severity,
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
        } => scan(path, format, fail_on),
        Command::Rules {
            action: RulesCommand::List { format },
        } => list(format),
        Command::Rules {
            action: RulesCommand::Explain { rule_id, format },
        } => explain(&rule_id, format),
    })
}

fn scan(path: PathBuf, format: Format, fail_on: Severity) -> u8 {
    let rules = sorobanshield_rules::active_rules();
    let (_project, outcome) = match discover_and_scan(&path, &rules) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            return EXIT_FAILURE;
        }
    };
    let ids = rules.iter().map(|r| r.meta().id).collect();
    let report = Report::new(
        env!("CARGO_PKG_VERSION"),
        &path.display().to_string(),
        ids,
        outcome,
    );
    match format {
        Format::Json => println!("{}", report.to_json()),
        Format::Text => print!("{}", report.to_terminal()),
    }
    if report.has_errors() {
        eprintln!(
            "error: {} file(s) could not be analyzed; the scan is incomplete",
            report.scan.errors.len()
        );
        EXIT_FAILURE
    } else if report.exceeds(fail_on) {
        EXIT_FINDINGS
    } else {
        EXIT_OK
    }
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
