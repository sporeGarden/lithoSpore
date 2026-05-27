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
pub mod domain_profile;
pub mod livespore;
pub mod scope;
pub mod tarball;
pub mod validation;
