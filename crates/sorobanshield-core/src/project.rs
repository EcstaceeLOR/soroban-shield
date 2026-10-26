use crate::rule::relative_display;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

const SKIPPED_DIRS: &[&str] = &["target", "node_modules", ".git", "vendor", "dist", "build"];

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("path does not exist: {0}")]
    NotFound(String),
    #[error("path is not a directory: {0}")]
    NotADirectory(String),
    #[error("no Cargo.toml found in {0}; run the scan from a Rust/Soroban project root")]
    MissingManifest(String),
    #[error("cannot read {path}: {reason}")]
    Unreadable { path: String, reason: String },
    #[error("malformed manifest {path}: {reason}")]
    MalformedManifest { path: String, reason: String },
    #[error(
        "{0} does not look like a Soroban project: no Cargo.toml declares a soroban-sdk dependency"
    )]
    NotSoroban(String),
    #[error("no Rust source files found under {0}")]
    NoSources(String),
}

/// A parsed `Cargo.toml` found in the project.
#[derive(Debug)]
pub struct Manifest {
    /// Path relative to the scan root, `/`-separated.
    pub path: String,
    pub table: toml::Table,
}

#[derive(Debug)]
pub struct Project {
    pub root: PathBuf,
    pub sources: Vec<PathBuf>,
    pub manifests: Vec<Manifest>,
}

fn declares_soroban(table: &toml::Table) -> bool {
    fn has(deps: Option<&toml::Value>) -> bool {
        deps.and_then(|d| d.as_table())
            .is_some_and(|t| t.contains_key("soroban-sdk"))
    }
    if ["dependencies", "dev-dependencies"]
        .iter()
        .any(|k| has(table.get(*k)))
    {
        return true;
    }
    table
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .is_some_and(|d| has(Some(d)))
}

fn skipped(name: &str) -> bool {
    SKIPPED_DIRS.contains(&name)
}

impl Project {
    pub fn discover(root: &Path) -> Result<Project, ProjectError> {
        Project::discover_with(root, &[])
    }

    /// `exclude` holds normalized project-relative path prefixes to skip.
    pub fn discover_with(root: &Path, exclude: &[String]) -> Result<Project, ProjectError> {
        let shown = root.display().to_string();
        if !root.exists() {
            return Err(ProjectError::NotFound(shown));
        }
        if !root.is_dir() {
            return Err(ProjectError::NotADirectory(shown));
        }
        if !root.join("Cargo.toml").is_file() {
            return Err(ProjectError::MissingManifest(shown));
        }

        let mut soroban = false;
        let mut sources = Vec::new();
        let mut manifests = Vec::new();
        let walker = WalkDir::new(root)
            .follow_links(false)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|e| {
                if e.depth() == 0 {
                    return true;
                }
                if e.file_type().is_dir() && skipped(&e.file_name().to_string_lossy()) {
                    return false;
                }
                let rel = relative_display(root, e.path());
                !exclude
                    .iter()
                    .any(|x| rel == *x || rel.starts_with(&format!("{x}/")))
            });
        for entry in walker {
            let entry = entry.map_err(|e| ProjectError::Unreadable {
                path: e
                    .path()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default(),
                reason: e.to_string(),
            })?;
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let rel = relative_display(root, path);
            if entry.file_name() == "Cargo.toml" {
                let text = std::fs::read_to_string(path).map_err(|e| ProjectError::Unreadable {
                    path: rel.clone(),
                    reason: e.to_string(),
                })?;
                let table: toml::Table =
                    text.parse()
                        .map_err(|e: toml::de::Error| ProjectError::MalformedManifest {
                            path: rel.clone(),
                            reason: e.message().to_string(),
                        })?;
                soroban |= declares_soroban(&table);
                manifests.push(Manifest { path: rel, table });
            } else if path.extension().is_some_and(|e| e == "rs") {
                sources.push(path.to_path_buf());
            }
        }

        if !soroban {
            return Err(ProjectError::NotSoroban(shown));
        }
        if sources.is_empty() {
            return Err(ProjectError::NoSources(shown));
        }
        Ok(Project {
            root: root.to_path_buf(),
            sources,
            manifests,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write(dir: &Path, rel: &str, text: &str) {
        let p = dir.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }

    const MANIFEST: &str =
        "[package]\nname=\"c\"\nversion=\"0.1.0\"\n[dependencies]\nsoroban-sdk = \"22\"\n";

    #[test]
    fn missing_path_is_an_error() {
        assert!(matches!(
            Project::discover(Path::new("/nonexistent/xyz")),
            Err(ProjectError::NotFound(_))
        ));
    }

    #[test]
    fn file_path_is_rejected() {
        let d = tempfile::tempdir().unwrap();
        write(d.path(), "a.rs", "");
        assert!(matches!(
            Project::discover(&d.path().join("a.rs")),
            Err(ProjectError::NotADirectory(_))
        ));
    }

    #[test]
    fn requires_manifest() {
        let d = tempfile::tempdir().unwrap();
        assert!(matches!(
            Project::discover(d.path()),
            Err(ProjectError::MissingManifest(_))
        ));
    }

    #[test]
    fn rejects_non_soroban_manifest() {
        let d = tempfile::tempdir().unwrap();
        write(
            d.path(),
            "Cargo.toml",
            "[package]\nname=\"x\"\nversion=\"0.1.0\"\n",
        );
        write(d.path(), "src/lib.rs", "");
        assert!(matches!(
            Project::discover(d.path()),
            Err(ProjectError::NotSoroban(_))
        ));
    }

    #[test]
    fn rejects_malformed_manifest() {
        let d = tempfile::tempdir().unwrap();
        write(d.path(), "Cargo.toml", "[package\n");
        assert!(matches!(
            Project::discover(d.path()),
            Err(ProjectError::MalformedManifest { .. })
        ));
    }

    #[test]
    fn skips_build_directories_and_non_rust_files() {
        let d = tempfile::tempdir().unwrap();
        write(d.path(), "Cargo.toml", MANIFEST);
        write(d.path(), "src/lib.rs", "");
        write(d.path(), "target/debug/gen.rs", "");
        write(d.path(), "node_modules/x/y.rs", "");
        write(d.path(), "notes.txt", "");
        let p = Project::discover(d.path()).unwrap();
        assert_eq!(p.sources.len(), 1);
        assert!(p.sources[0].ends_with("src/lib.rs"));
    }

    #[test]
    fn detects_workspace_member_dependency() {
        let d = tempfile::tempdir().unwrap();
        write(d.path(), "Cargo.toml", "[workspace]\nmembers=[\"c\"]\n");
        write(d.path(), "c/Cargo.toml", MANIFEST);
        write(d.path(), "c/src/lib.rs", "");
        assert!(Project::discover(d.path()).is_ok());
    }
}

#[cfg(test)]
mod exclude_tests {
    use super::*;
    use std::fs;

    #[test]
    fn exclude_prunes_directories_files_and_manifests() {
        let d = tempfile::tempdir().unwrap();
        let w = |rel: &str, text: &str| {
            let p = d.path().join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, text).unwrap();
        };
        w(
            "Cargo.toml",
            "[package]\nname=\"c\"\nversion=\"0.1.0\"\n[dependencies]\nsoroban-sdk=\"22\"\n",
        );
        w("src/lib.rs", "");
        w("gen/a.rs", "");
        w("gen/deep/b.rs", "");
        w("src/skip.rs", "");
        w("generated/c.rs", "");
        let p = Project::discover_with(d.path(), &["gen".into(), "src/skip.rs".into()]).unwrap();
        let rels: Vec<String> = p
            .sources
            .iter()
            .map(|s| relative_display(d.path(), s))
            .collect();
        assert_eq!(rels, vec!["generated/c.rs", "src/lib.rs"]);
    }
}
