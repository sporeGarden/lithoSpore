// SPDX-License-Identifier: AGPL-3.0-or-later

//! litho-core: domain-agnostic engine for lithoSpore guideStone verification.
//!
//! Provides the scope graph, validation harness, tolerance framework,
//! provenance chain, liveSpore tracking, capability-based discovery, shared
//! statistics, and data manifest types. The LTEE modules are the first
//! instance; any guideStone instance can reuse this engine via `scope.toml`.

pub mod braid;
pub mod discovery;
pub mod graph_checks;
pub mod harness;
pub mod manifest;
pub mod provenance;
pub mod pseudospore;
pub mod scope;
pub mod spore;
pub mod stats;
pub mod tolerance;
pub mod validation;

pub use braid::{BraidCheck, BraidComputation, CloneMutationCount, FermentBraid};
pub use discovery::{DiscoveryPath, PrimalListResponse};
pub use manifest::DataManifest;
pub use provenance::ProvenanceEntry;
pub use pseudospore::{PseudoSporeManifest, PseudoSporeScope, SporeStatus};
pub use scope::{ScopeManifest, ScopeModule};
pub use spore::LiveSporeEntry;
pub use tolerance::{Tolerance, ToleranceSet};
pub use validation::{
    ModuleResult, ParityReport, ParityResult, ParityStatus, TargetCoverage, Tier3Session,
    ValidationReport, ValidationStatus,
};
