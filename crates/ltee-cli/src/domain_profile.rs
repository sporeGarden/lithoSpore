// SPDX-License-Identifier: AGPL-3.0-or-later

//! Domain profile — re-exports from `pseudospore-core` plus litho CLI loaders.
//!
//! Domain-specific audit checks (PLUMED, GROMACS MDP, etc.) live in [`crate::audit::domain`].

pub(crate) use pseudospore_core::DomainProfile;

use std::path::Path;

/// Load a domain profile from a specific file path.
/// Returns `None` if the file doesn't exist or fails to parse.
#[must_use]
pub(crate) fn load_from_file(path: &Path) -> Option<DomainProfile> {
    DomainProfile::try_load(path)
}

/// Load a domain profile from `domain_profile.toml` at the given root.
/// Returns `None` if the file doesn't exist (graceful degradation).
#[must_use]
pub(crate) fn load_domain_profile(root: &Path) -> Option<DomainProfile> {
    DomainProfile::from_spore_root(root)
}
