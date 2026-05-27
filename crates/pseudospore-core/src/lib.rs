// SPDX-License-Identifier: AGPL-3.0-or-later

//! pseudospore-core: domain-agnostic spore envelope primitives.
//!
//! This crate provides the shared types and logic for pseudoSpore/lithoSpore
//! lifecycle operations. Both the litho CLI (gardens/lithoSpore) and biomeOS
//! (primals/biomeOS) depend on this crate for envelope parsing, BLAKE3
//! manifests, liveSpore schema, and tarball creation.
//!
//! Domain-specific science (PLUMED, GROMACS, LTEE, etc.) does NOT belong here.
//! See `SPORE_OWNERSHIP_MATRIX.md` for the three-way ownership split.

pub mod blake3_manifest;
pub mod braid_envelope;
pub mod domain_profile;
pub mod envelope;
pub mod error;
pub mod livespore;
pub mod receipts;
pub mod scope;
pub mod tarball;
pub mod validation;

pub use blake3_manifest::Blake3Manifest;
pub use braid_envelope::FermentTranscript;
pub use domain_profile::{
    AuditConfig, AuditDomainFlags, AuditValidationFlags, CheckCommand, ClaimValidator, ClaimZone,
    DerivationConfig, DerivationContract, DomainProfile, EntityGroup, FigurePlot, FiguresConfig,
    ProfileModule, SimTimeConfig, TranslationConfig,
};
pub use envelope::{EnvelopeValidation, PseudoSporeEnvelope};
pub use error::SporeError;
pub use livespore::{LiveSporeDoc, ValidationEntry};
pub use receipts::{
    ChecksumEntry, EnvironmentReceipt, compute_checksums, format_checksums, parse_checksums,
};
pub use scope::ScopeDoc;
pub use validation::ValidationDoc;
