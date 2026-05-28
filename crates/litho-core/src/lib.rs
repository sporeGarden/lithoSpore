// SPDX-License-Identifier: AGPL-3.0-or-later

//! litho-core: domain-agnostic engine for lithoSpore guideStone verification.
//!
//! Provides the scope graph, validation harness, tolerance framework,
//! provenance chain, liveSpore tracking, capability-based discovery, shared
//! statistics, and data manifest types. The LTEE modules are the first
//! instance; any guideStone instance can reuse this engine via `scope.toml`.

pub mod braid;
pub mod discovery;
pub mod env_vars;
pub mod harness;
pub mod manifest;
pub mod provenance;
pub mod scope;
pub mod spore;
pub mod stats;
pub mod tolerance;
pub mod validation;

// ── LTEE domain constants ──────────────────────────────────────────
/// *E. coli* K-12 MG1655 reference genome length in base pairs.
pub const E_COLI_K12_MG1655_BP: f64 = 4_629_812.0;
/// Number of replicate populations in the LTEE.
pub const LTEE_N_POPULATIONS: u64 = 12;

pub use braid::{BraidCheck, BraidComputation, CloneMutationCount, FermentBraid};
pub use discovery::{DiscoveryPath, PrimalListResponse};
pub use manifest::DataManifest;
pub use provenance::ProvenanceEntry;
pub use scope::{ScopeManifest, ScopeModule};
pub use spore::LiveSporeEntry;
pub use tolerance::{Tolerance, ToleranceSet};
pub use validation::{
    ModuleResult, ParityReport, ParityResult, ParityStatus, TargetCoverage, Tier3Session,
    ValidationReport, ValidationStatus,
};
