// SPDX-License-Identifier: AGPL-3.0-or-later

//! Typed error hierarchy for litho-core operations.

use std::path::PathBuf;

/// Errors from loading, parsing, or interacting with lithoSpore chassis components.
#[derive(Debug, thiserror::Error)]
pub enum LithoError {
    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("failed to parse {path}: {detail}")]
    Parse { path: PathBuf, detail: String },

    #[error("serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("discovery failed: {0}")]
    Discovery(String),

    #[error("{method}: {detail}")]
    Rpc { method: String, detail: String },
}
