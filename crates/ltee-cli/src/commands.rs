// SPDX-License-Identifier: AGPL-3.0-or-later

//! CLI subcommand definitions (clap derive).
//!
//! Extracted from `main.rs` to keep the dispatch module focused on routing.

use clap::{Parser, Subcommand};

fn parse_module_status(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(format!("Expected NAME=STATUS, got '{s}'"));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

#[derive(Parser)]
#[command(
    name = "litho",
    about = "lithoSpore — guideStone verification chassis",
    version,
    long_about = "Self-contained, scope-driven validation artifact.\nCurrent instance: LTEE (Long-Term Evolution Experiment).\nSee https://github.com/sporeGarden/lithoSpore"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run all science modules (scope-driven) and produce structured validation output
    Validate {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        #[arg(long)]
        json: bool,

        /// Max tier: 1 = Python only, 2 = Rust (default), 3 = Rust + NUCLEUS provenance.
        #[arg(long, default_value = "2")]
        max_tier: u8,

        /// Write provenance artifacts (results.json + provenance.toml) to this directory.
        /// Follows projectFOUNDATION Thread 10 conventions.
        #[arg(long)]
        provenance_dir: Option<String>,
    },

    /// Cross-tier parity check: run Tier 1 and Tier 2 side-by-side and compare results
    Parity {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        #[arg(long)]
        json: bool,
    },

    /// Re-fetch datasets from source URIs and re-validate
    Refresh {
        #[arg(long, default_value = ".")]
        artifact_root: String,
    },

    /// Show artifact status: version, tier support, module availability
    Status {
        #[arg(long, default_value = ".")]
        artifact_root: String,
    },

    /// Show liveSpore deployment history
    Spore {
        #[arg(long, default_value = ".")]
        artifact_root: String,
    },

    /// Verify data integrity: rehash local files against manifest, and
    /// optionally probe upstream source URIs for changes when online
    Verify {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate scientific visualizations for all modules
    Visualize {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        /// Output format: svg, json, dashboard, baselines (Barrick Lab baseline validation)
        #[arg(long, default_value = "json")]
        format: String,

        /// Output directory for generated figures (--format svg)
        #[arg(long, default_value = "figures")]
        output: String,
    },

    /// Validate artifact integrity: expected JSONs, data dirs, binaries, papers
    SelfTest {
        #[arg(long, default_value = ".")]
        artifact_root: String,
    },

    /// Report which validation tier is achievable on this machine
    Tier {
        #[arg(long, default_value = ".")]
        artifact_root: String,
    },

    /// Assemble the USB artifact directory (replaces scripts/assemble-usb.sh)
    Assemble {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        /// Target directory for the assembled artifact
        #[arg(long, default_value = "usb-staging")]
        target: String,

        #[arg(long)]
        skip_python: bool,

        #[arg(long)]
        skip_fetch: bool,

        #[arg(long)]
        skip_build: bool,

        #[arg(long)]
        dry_run: bool,
    },

    /// Fetch datasets from source URIs (replaces scripts/fetch_*.sh)
    Fetch {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        /// Fetch a specific dataset by ID or module name
        #[arg(long)]
        dataset: Option<String>,

        /// Fetch all datasets
        #[arg(long)]
        all: bool,

        /// Fetch full upstream data (SRA reads, complete archives) instead of summary stats.
        /// Requires SRA toolkit for genomic datasets. Can be 10s–100s of GB.
        #[arg(long)]
        full: bool,
    },

    /// Run fault injection tests against the artifact (replaces scripts/chaos-test.sh)
    ChaosTest {
        #[arg(long, default_value = ".")]
        artifact_root: String,
    },

    /// Simulate local deployment: assemble, verify, validate (replaces scripts/deploy-test-local.sh)
    DeployTest {
        #[arg(long, default_value = ".")]
        artifact_root: String,
    },

    /// Generate a TOML deployment report combining self-test, validate, verify
    DeployReport {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        /// Deployment pattern label (e.g. container-airgap, vps-spore, usb-local)
        #[arg(long, default_value = "local")]
        pattern: String,
    },

    /// Grow: germinate the USB artifact into a full development environment
    Grow {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        /// Target directory for the cloned source tree
        #[arg(long, default_value = ".")]
        target: String,

        /// Also provision a benchScale VM for isolated validation
        #[arg(long)]
        vm: bool,

        /// Deploy via Docker/Podman container (works on any OS)
        #[arg(long)]
        container: bool,

        /// Also clone the full ecoPrimals ecosystem
        #[arg(long)]
        ecosystem: bool,

        /// Skip building from source
        #[arg(long)]
        skip_build: bool,

        /// Skip fetching upstream datasets
        #[arg(long)]
        skip_fetch: bool,
    },

    /// Ingest a pseudoSpore: validate structure, import braids, register.
    ///
    /// Prefer `biomeos nucleus ingest` when NUCLEUS is available.
    /// This command is the offline/airgapped fallback path.
    ///
    /// Both paths validate via pseudospore-core; NUCLEUS adds provenance trio
    /// registration.
    IngestPseudospore {
        /// Path to the pseudoSpore directory
        path: String,

        #[arg(long, default_value = ".")]
        artifact_root: String,

        /// Verify BLAKE3 checksums after structural validation
        #[arg(long)]
        verify: bool,
    },

    /// Fetch a pseudoSpore from a remote URL (hosted gallery or direct tarball).
    ///
    /// Downloads, extracts, validates via pseudospore-core, and optionally
    /// chains into `ingest-pseudospore` for registry and braid import.
    FetchPseudospore {
        /// URL to download (tarball: .tar.gz)
        #[arg(long, alias = "from")]
        url: String,

        /// Output directory for extracted pseudoSpore
        #[arg(long, default_value = ".")]
        output: String,

        #[arg(long, default_value = ".")]
        artifact_root: String,

        /// After fetch+validate, automatically ingest into lithoSpore registry
        #[arg(long)]
        ingest: bool,
    },

    /// Emit a pseudoSpore: assemble standard directory structure from module outputs.
    /// Works for any spring — driven by `domain_profile.toml` when provided.
    ///
    /// Use `--from-dir` to re-emit from an existing pseudoSpore directory (reads
    /// name/version/origin from its `scope.toml`). This is the delegation path
    /// used by nest-validate and other springs.
    EmitPseudospore {
        /// Artifact name (required unless --from-dir is set)
        #[arg(long, required_unless_present = "from_dir")]
        name: Option<String>,

        /// Artifact version (required unless --from-dir is set)
        #[arg(long, required_unless_present = "from_dir")]
        version: Option<String>,

        /// Origin spring/repo path (e.g., "ecoPrimals/springs/hotSpring")
        #[arg(long, default_value = "")]
        origin: String,

        /// Source spring name (e.g., "hotSpring", "groundSpring"). Auto-inferred from origin if omitted.
        #[arg(long)]
        spring: Option<String>,

        /// Output directory (pseudoSpore dir created inside)
        #[arg(long, default_value = ".")]
        output: String,

        /// Directory containing output files to include
        #[arg(long)]
        outputs: Option<String>,

        /// Directory containing config files to include
        #[arg(long)]
        configs: Option<String>,

        /// Directory containing braid JSON files to include
        #[arg(long)]
        braids: Option<String>,

        /// Directory containing raw data files (HILLS, topology) for zero-trust verification
        #[arg(long)]
        data: Option<String>,

        /// Path to a `domain_profile.toml` — drives domain-specific emit logic.
        /// Per `SPORE_OWNERSHIP_MATRIX.md`: each spring provides its own profile.
        #[arg(long, alias = "domain-profile")]
        profile: Option<String>,

        /// Re-emit from an existing pseudoSpore directory. Reads name, version,
        /// and origin from its `scope.toml`. Use for delegation from nest-validate.
        #[arg(long)]
        from_dir: Option<String>,
    },

    /// Pre-handoff audit: check config fidelity, translation, completeness, versioning
    Audit {
        /// Path to the pseudoSpore or lithoSpore proof/ directory
        #[arg(long, default_value = ".")]
        path: String,

        /// Show fix suggestions for each finding
        #[arg(long)]
        verbose: bool,

        /// Emit structured JSON report (guideStone validation format)
        #[arg(long)]
        json: bool,
    },

    /// Promote a pseudoSpore to a lithoSpore deployment chassis
    Promote {
        /// Path to the pseudoSpore directory
        #[arg(long)]
        pseudospore: String,

        /// Output directory (lithoSpore dir created inside)
        #[arg(long, default_value = ".")]
        output: String,

        /// Path to Tier 2 Rust crate to compile and include
        #[arg(long)]
        tier2_crate: Option<String>,

        /// Path to Tier 1 Python validation script to include
        #[arg(long)]
        tier1_script: Option<String>,

        /// Override the lithoSpore version (default: workspace package version)
        #[arg(long)]
        version: Option<String>,
    },

    /// Pack a pseudoSpore directory into a distributable .tar.gz tarball.
    /// Only present files are included; external data is excluded.
    PackPseudospore {
        /// Path to the pseudoSpore directory
        path: String,

        /// Output tarball path (default: `DIR.tar.gz` alongside the directory)
        #[arg(long)]
        output: Option<String>,

        /// Patterns for external files to exclude (default: data/, structures/, topologies/)
        #[arg(long)]
        external: Vec<String>,
    },

    /// Unpack a pseudoSpore .tar.gz tarball into a directory and optionally validate.
    UnpackPseudospore {
        /// Path to the .tar.gz tarball
        tarball: String,

        /// Output directory (pseudoSpore extracted inside)
        #[arg(long, default_value = ".")]
        output: String,

        /// Run envelope validation after extraction
        #[arg(long)]
        validate: bool,
    },

    /// Populate a pseudoSpore's validation.json with module results.
    ///
    /// Use `--results` to supply a JSON file containing an array of module results,
    /// or use `--module` to set individual module statuses inline.
    PopulateValidation {
        /// Path to the pseudoSpore directory
        path: String,

        /// Path to a JSON file containing module results (array of `ValidationModule`)
        #[arg(long)]
        results: Option<String>,

        /// Set a module status inline: --module name=PASS
        #[arg(long = "module", value_parser = parse_module_status)]
        modules: Vec<(String, String)>,
    },

    /// Promote a pseudoSpore from PENDING to COMPLETE after all modules pass.
    PromoteSpore {
        /// Path to the pseudoSpore directory (must contain validation.json)
        path: String,

        /// Artifact root containing pseudospores/registry.toml (default: current dir)
        #[arg(long, default_value = ".")]
        artifact_root: String,
    },

    /// Show registry status dashboard for all pseudoSpores
    SporeStatus {
        #[arg(long, default_value = ".")]
        artifact_root: String,

        /// Emit structured JSON report
        #[arg(long)]
        json: bool,
    },

    /// Translate config file indices between domain and computation frames
    TranslateConfig {
        /// Path to `index_map.toml`
        #[arg(long)]
        index_map: String,

        /// Path to the config file to translate (e.g. plumed.dat)
        #[arg(long)]
        config: String,

        /// Target frame: 'domain' (PDB numbering) or 'computation' (runtime indices)
        #[arg(long, default_value = "domain")]
        frame: String,

        /// Output file path (prints to stdout if not specified)
        #[arg(long)]
        output: Option<String>,
    },
}
