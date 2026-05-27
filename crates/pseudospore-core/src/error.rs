// SPDX-License-Identifier: AGPL-3.0-or-later

//! Typed error hierarchy for pseudospore-core operations.

use std::path::PathBuf;

/// Errors from loading or validating pseudoSpore components.
#[derive(Debug, thiserror::Error)]
pub enum SporeError {
    #[error("file not found: {0}")]
    NotFound(PathBuf),

    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to parse {path}: {detail}")]
    Parse { path: PathBuf, detail: String },

    #[error("validation failed: {0}")]
    Validation(String),

    #[error("checksum mismatch for {path}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        path: String,
        expected: String,
        actual: String,
    },

    #[error("{0}")]
    Other(String),
}

impl From<String> for SporeError {
    fn from(s: String) -> Self {
        Self::Other(s)
    }
}
