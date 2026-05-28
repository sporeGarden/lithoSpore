// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho grow` — germinate a USB artifact into a full development environment.
//!
//! The USB carries everything needed for offline validation (Tier 1 + 2).
//! When plugged into a clean system with internet, `litho grow` clones the
//! source repo, installs the Rust toolchain, builds from source, fetches
//! all upstream data, and validates — turning the seed into a full tree.
//!
//! Stages:
//!   1. Clone source repo (from scope.toml [source] metadata)
//!   2. Detect or install Rust toolchain
//!   3. Build from source (cargo build --release)
//!   4. Seed data from USB into cloned tree
//!   5. Fetch remaining datasets
//!   6. Validate (Tier 1 + Tier 2)
//!   7. Optionally provision a benchScale VM

mod deploy;
mod stages;
mod util;

use deploy::{stage_container, stage_vm};
use stages::{
    load_source_metadata, stage_build, stage_clone, stage_fetch, stage_seed_data, stage_toolchain,
    stage_validate,
};
use std::path::{Path, PathBuf};
use util::step;

/// Runtime mode flags for grow.
pub(crate) struct GrowModeFlags {
    pub vm: bool,
    pub container: bool,
    pub ecosystem: bool,
}

/// Skip flags for grow pipeline stages.
pub(crate) struct GrowSkipFlags {
    pub build: bool,
    pub fetch: bool,
}

/// Options for germinating a USB artifact into a dev environment.
pub(crate) struct GrowOptions<'a> {
    pub artifact_root: &'a str,
    pub target: &'a str,
    pub mode: GrowModeFlags,
    pub skip: GrowSkipFlags,
}

pub(crate) fn run(opts: &GrowOptions<'_>) {
    let GrowOptions {
        artifact_root,
        target,
        mode,
        skip,
    } = opts;
    let vm = mode.vm;
    let container = mode.container;
    let ecosystem = mode.ecosystem;
    let skip_build = skip.build;
    let skip_fetch = skip.fetch;
    let root = Path::new(artifact_root);
    let target_path = PathBuf::from(target);

    // Container-only mode: build OCI image and validate inside it.
    // No clone/build/fetch needed — the USB artifact is self-contained.
    if container {
        stage_container(root);
        return;
    }

    println!("=== lithoSpore grow — germinating hypogeal cotyledon ===");
    println!("  Artifact: {artifact_root}");
    println!("  Target:   {target}");
    println!();

    let scope = load_source_metadata(root);

    // Stage 1: Clone
    stage_clone(root, &target_path, &scope, ecosystem);

    // Stage 2: Toolchain
    if !skip_build {
        stage_toolchain(&scope);
    }

    // Stage 3: Build
    if !skip_build {
        stage_build(&target_path, &scope);
    }

    // Stage 4: Seed data from USB
    stage_seed_data(root, &target_path);

    // Stage 5: Fetch
    if !skip_fetch {
        stage_fetch(&target_path);
    }

    // Stage 6: Validate
    stage_validate(&target_path);

    // Stage 7: VM (optional)
    if vm {
        stage_vm(root);
    }

    println!();
    step("Germination Complete");
    println!("  Full source tree:  {target}");
    println!(
        "  Build artifacts:   {}/target/release/",
        target_path.display()
    );
    println!();
    println!("  To develop:  cd {target} && cargo test --workspace");
    println!("  To validate: cd {target} && cargo run --release --bin litho -- validate");
    if ecosystem {
        let eco = target_path.parent().unwrap_or(&target_path);
        println!("  Ecosystem:   {}", eco.display());
    }
}
