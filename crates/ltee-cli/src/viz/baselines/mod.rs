// SPDX-License-Identifier: AGPL-3.0-or-later

//! Barrick Lab baseline tool `DataBinding` adapters.
//!
//! Each adapter reads a tool's `reference_data.json` and produces
//! petalTongue-compatible `DataBinding` JSON values reproducing
//! the tool's key visualization patterns.

mod breseq_adapter;
mod cryptkeeper_adapter;
mod efm_adapter;
mod marker_divergence_adapter;
mod ostir_adapter;
mod plannotate_adapter;
mod rna_mi_adapter;

pub use breseq_adapter::breseq;
pub use cryptkeeper_adapter::cryptkeeper;
pub use efm_adapter::efm;
pub use marker_divergence_adapter::marker_divergence;
pub use ostir_adapter::ostir;
pub use plannotate_adapter::plannotate;
pub use rna_mi_adapter::rna_mi;
