use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use lintdiff_engine::{
    parse_cargo_analysis_with_repo_root, parse_cargo_messages, parse_cargo_messages_with_status,
    CargoAnalysis, CargoDiagnosticStream, Diagnostic,
};
use lintdiff_types::{inventory::Inventory, LintdiffConfig, Report};
use serde_json::to_vec_pretty;
use thiserror::Error;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

#[derive(Debug, Error)]
pub enum AppIoError {
    #[error("failed to read file {path}: {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write file {path}: {source}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse config toml: {source}")]
    ParseConfig {
        #[source]
        source: toml::de::Error,
    },
    #[error("failed to parse diagnostics: {msg}")]
    DiagnosticsParse { msg: String },
    #[error("failed to serialize report JSON: {source}")]
    Serialize {
        #[source]
        source: serde_json::Error,
    },
}

pub fn load_config(
    repo_root: &Path,
    explicit: Option<&Path>,
) -> Result<LintdiffConfig, AppIoError> {
    let path = if let Some(p) = explicit {
        Some(p.to_path_buf())
    } else {
        let candidate = repo_root.join("lintdiff.toml");
        if candidate.exists() {
            Some(candidate)
        } else {
            None
        }
    };

    let Some(path) = path else {
        return Ok(LintdiffConfig::default());
    };

    let raw = std::fs::read_to_string(&path).map_err(|e| AppIoError::ReadFile {
        path: path.clone(),
        source: e,
    })?;
    let cfg: LintdiffConfig =
        toml::from_str(&raw).map_err(|e| AppIoError::ParseConfig { source: e })?;
    Ok(cfg)
}

pub fn parse_diagnostics<R: BufRead>(reader: R) -> Result<Vec<Diagnostic>, AppIoError> {
    parse_cargo_messages(reader).map_err(|e| AppIoError::DiagnosticsParse { msg: e.to_string() })
}

pub fn parse_diagnostics_with_status<R: BufRead>(
    reader: R,
) -> Result<CargoDiagnosticStream, AppIoError> {
    parse_cargo_messages_with_status(reader)
        .map_err(|e| AppIoError::DiagnosticsParse { msg: e.to_string() })
}

pub fn acquire_diagnostics(path: Option<&Path>) -> Result<Option<Vec<Diagnostic>>, AppIoError> {
    Ok(acquire_diagnostics_with_status(path)?.map(|stream| stream.diagnostics))
}

pub fn acquire_diagnostics_with_status(
    path: Option<&Path>,
) -> Result<Option<CargoDiagnosticStream>, AppIoError> {
    if let Some(p) = path {
        let f = std::fs::File::open(p).map_err(|e| AppIoError::ReadFile {
            path: p.to_path_buf(),
            source: e,
        })?;
        let reader = BufReader::new(f);
        let stream = parse_diagnostics_with_status(reader)?;
        return Ok(Some(stream));
    }

    let mut buf = String::new();
    io::stdin()
        .read_to_string(&mut buf)
        .map_err(|e| AppIoError::ReadFile {
            path: PathBuf::from("<stdin>"),
            source: e,
        })?;

    if buf.trim().is_empty() {
        return Ok(None);
    }

    let reader = BufReader::new(buf.as_bytes());
    let stream = parse_diagnostics_with_status(reader)?;
    Ok(Some(stream))
}

pub fn acquire_cargo_analysis(
    path: Option<&Path>,
    repo_root: &Path,
) -> Result<CargoAnalysis, AppIoError> {
    let mut buf = String::new();
    if let Some(path) = path {
        buf = std::fs::read_to_string(path).map_err(|source| AppIoError::ReadFile {
            path: path.to_path_buf(),
            source,
        })?;
    } else {
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(|source| AppIoError::ReadFile {
                path: PathBuf::from("<stdin>"),
                source,
            })?;
    }

    parse_cargo_analysis_with_repo_root(BufReader::new(buf.as_bytes()), Some(repo_root)).map_err(
        |error| AppIoError::DiagnosticsParse {
            msg: error.to_string(),
        },
    )
}

pub fn write_report_json(report: &Report, path: &Path) -> Result<(), AppIoError> {
    let bytes = to_vec_pretty(report).map_err(|e| AppIoError::Serialize { source: e })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppIoError::WriteFile {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }
    std::fs::write(path, bytes).map_err(|e| AppIoError::WriteFile {
        path: path.to_path_buf(),
        source: e,
    })
}

pub fn write_inventory_json(inventory: &Inventory, path: &Path) -> Result<(), AppIoError> {
    let bytes = to_vec_pretty(inventory).map_err(|source| AppIoError::Serialize { source })?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| AppIoError::WriteFile {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    std::fs::write(path, bytes).map_err(|source| AppIoError::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}

pub fn write_text(path: &Path, contents: &str) -> Result<(), AppIoError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppIoError::WriteFile {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }
    std::fs::write(path, contents).map_err(|e| AppIoError::WriteFile {
        path: path.to_path_buf(),
        source: e,
    })
}

pub fn now_rfc3339() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}
