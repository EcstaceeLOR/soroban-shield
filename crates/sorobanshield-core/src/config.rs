use crate::finding::Severity;
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("cannot read config file {path}: {reason}")]
    Read { path: String, reason: String },
    #[error("invalid config file {path}: {reason}")]
    Parse { path: String, reason: String },
    #[error("invalid configuration: {0}")]
    Invalid(String),
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ScanConfig {
    /// Project-relative paths (files or directories) to skip.
    pub exclude: Vec<String>,
    /// Minimum severity that makes the scan exit with code 1.
    pub fail_on: Option<Severity>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct RulesConfig {
    /// Rule IDs that should not run.
    pub disable: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct Config {
    pub scan: ScanConfig,
    pub rules: RulesConfig,
}

fn normalize_exclude(raw: &str) -> Result<String, ConfigError> {
    let bad = |why: &str| ConfigError::Invalid(format!("[scan].exclude entry '{raw}' {why}"));
    let mut p = raw.trim();
    while let Some(rest) = p.strip_prefix("./") {
        p = rest;
    }
    let p = p.trim_end_matches('/');
    if p.is_empty() || p == "." {
        return Err(bad("is empty or refers to the whole project"));
    }
    if p.starts_with('/') || p.contains(":\\") || p.contains('\\') {
        return Err(bad("must be a relative path using '/' separators"));
    }
    if p.split('/').any(|c| c == ".." || c.is_empty()) {
        return Err(bad("must not contain '..' or empty path components"));
    }
    if p.contains(['*', '?', '[', ']', '{', '}']) {
        return Err(bad(
            "contains glob characters; globs are not supported, list path prefixes instead",
        ));
    }
    Ok(p.to_string())
}

impl Config {
    pub fn parse(text: &str, shown_path: &str) -> Result<Config, ConfigError> {
        let mut cfg: Config = toml::from_str(text).map_err(|e| ConfigError::Parse {
            path: shown_path.to_string(),
            reason: e.message().to_string(),
        })?;
        cfg.scan.exclude = cfg
            .scan
            .exclude
            .iter()
            .map(|e| normalize_exclude(e))
            .collect::<Result<_, _>>()?;
        Ok(cfg)
    }

    pub fn load(path: &Path) -> Result<Config, ConfigError> {
        let shown = path.display().to_string();
        let text = std::fs::read_to_string(path).map_err(|e| ConfigError::Read {
            path: shown.clone(),
            reason: e.to_string(),
        })?;
        Config::parse(&text, &shown)
    }

    /// Checks `[rules].disable` against the rules that can actually run and normalizes IDs.
    pub fn validate_rules(&mut self, runnable_ids: &[String]) -> Result<(), ConfigError> {
        let mut disabled = Vec::new();
        for id in &self.rules.disable {
            let up = id.trim().to_ascii_uppercase();
            if !runnable_ids.contains(&up) {
                return Err(ConfigError::Invalid(format!(
                    "[rules].disable names unknown or non-runnable rule '{id}'; runnable rules: {}",
                    runnable_ids.join(", ")
                )));
            }
            if !disabled.contains(&up) {
                disabled.push(up);
            }
        }
        if disabled.len() == runnable_ids.len() {
            return Err(ConfigError::Invalid(
                "[rules].disable would disable every rule, leaving nothing to scan with".into(),
            ));
        }
        self.rules.disable = disabled;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids() -> Vec<String> {
        vec!["SS-001".into(), "SS-002".into()]
    }

    #[test]
    fn empty_config_is_default() {
        assert_eq!(Config::parse("", "c.toml").unwrap(), Config::default());
    }

    #[test]
    fn parses_all_fields_and_normalizes_excludes() {
        let c = Config::parse(
            "[scan]\nexclude = [\"./vendor/\", \"tests/fixtures\"]\nfail_on = \"medium\"\n[rules]\ndisable = [\"ss-002\"]\n",
            "c.toml",
        )
        .unwrap();
        assert_eq!(c.scan.exclude, vec!["vendor", "tests/fixtures"]);
        assert_eq!(c.scan.fail_on, Some(Severity::Medium));
        let mut c = c;
        c.validate_rules(&ids()).unwrap();
        assert_eq!(c.rules.disable, vec!["SS-002"]);
    }

    #[test]
    fn rejects_unknown_keys_at_every_level() {
        assert!(Config::parse("[scan]\nexlude = []\n", "c").is_err());
        assert!(Config::parse("[output]\nformat = \"json\"\n", "c").is_err());
        assert!(Config::parse("bogus = 1\n", "c").is_err());
    }

    #[test]
    fn rejects_bad_values() {
        assert!(Config::parse("[scan]\nfail_on = \"urgent\"\n", "c").is_err());
        assert!(Config::parse("[scan]\nfail_on = 3\n", "c").is_err());
        assert!(Config::parse("[scan\n", "c").is_err());
    }

    #[test]
    fn rejects_unsafe_or_unsupported_excludes() {
        for bad in ["/etc", "../x", "a/../b", "src/*.rs", "", ".", "a\\\\b"] {
            let t = format!("[scan]\nexclude = [\"{bad}\"]\n");
            assert!(Config::parse(&t, "c").is_err(), "{bad} should be rejected");
        }
    }

    #[test]
    fn rejects_unknown_rule_ids_and_disabling_everything() {
        let mut c = Config::parse("[rules]\ndisable = [\"SS-999\"]\n", "c").unwrap();
        assert!(c.validate_rules(&ids()).is_err());
        let mut all = Config::parse("[rules]\ndisable = [\"SS-001\", \"SS-002\"]\n", "c").unwrap();
        assert!(all.validate_rules(&ids()).is_err());
    }
}
