// SPDX-License-Identifier: AGPL-3.0-or-later

//! litho-core: shared types for lithoSpore Targeted `GuideStone` modules.
//!
//! Provides the validation harness, tolerance framework, provenance chain,
//! liveSpore tracking, and data manifest types used by all 7 LTEE modules.

pub mod manifest;
pub mod provenance;
pub mod tolerance;
pub mod validation;
pub mod spore;

/// Re-export key types for ergonomic use by module crates.
pub use manifest::DataManifest;
pub use provenance::ProvenanceEntry;
pub use tolerance::{Tolerance, ToleranceSet};
pub use validation::{ModuleResult, ValidationReport, ValidationStatus};
pub use spore::LiveSporeEntry;
