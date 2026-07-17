// SPDX-License-Identifier: AGPL-3.0-or-later

//! Domain profile — declarative config for domain-specific pseudoSpore behavior.
//!
//! A `domain_profile.toml` tells emit/audit/promote what domain-specific logic to
//! apply. When absent, only core (domain-agnostic) checks run. Declares WHAT, not HOW.
//! Domain-specific implementations live in their respective springs.
//!
//! TOML section parsers live in the `parse` submodule.

mod parse;

use std::path::Path;

/// Parsed domain profile configuration.
#[derive(Debug, Clone)]
pub struct DomainProfile {
    /// Profile identifier (matches filename stem or domain name).
    pub id: String,
    pub version: String,
    /// Owning spring / garden (e.g. `lithoSpore`, `biomeOS`).
    pub spring: Option<String>,
    /// External tools required by this domain (e.g. `gromacs`, `plumed`).
    pub tools: Vec<String>,
    pub modules: Vec<ProfileModule>,
    pub check_commands: Vec<CheckCommand>,
    pub translation: Option<TranslationConfig>,
    pub derivation: Option<DerivationConfig>,
    pub figures: Option<FiguresConfig>,
    pub audit: Option<AuditConfig>,
    pub simulation_time: Option<SimTimeConfig>,
    /// Optional RMSD acceptance limits for promote / expected stubs.
    pub tolerances: Option<TolerancesConfig>,
}

/// A module declared in the domain profile (`[[module]]`).
#[derive(Debug, Clone)]
pub struct ProfileModule {
    pub name: String,
    pub description: String,
    /// Shell command invoked to validate this module.
    pub check_command: String,
}

/// A domain-specific check command (`[[check]]`).
#[derive(Debug, Clone)]
pub struct CheckCommand {
    pub name: String,
    pub command: String,
    /// Expected process exit code (0 = success).
    pub expected_exit: i32,
}

/// Index translation settings (`[translation]`).
#[derive(Debug, Clone, Default)]
pub struct TranslationConfig {
    pub enabled: bool,
    /// Coordinate frame used in domain indices (e.g. residue numbering).
    pub domain_frame: String,
    /// Coordinate frame used in simulation topology files.
    pub computation_frame: String,
    /// Topology file format (e.g. `gro`, `pdb`).
    pub topology_format: String,
    pub entity_groups: Vec<EntityGroup>,
}

/// Entity group for domain↔computation index mapping.
#[derive(Debug, Clone)]
pub struct EntityGroup {
    pub name: String,
    /// Atom names belonging to this group.
    pub atoms: Vec<String>,
    /// Residue names or patterns to include when mapping indices.
    pub residue_filter: Vec<String>,
}

/// Derivation / reproduction contracts (`[derivation]`).
#[derive(Debug, Clone, Default)]
pub struct DerivationConfig {
    pub tool: String,
    /// Glob patterns to locate derivable input files.
    pub find_paths: Vec<String>,
    pub contracts: Vec<DerivationContract>,
}

/// Single derivation contract (`[[derivation.contract]]`).
#[derive(Debug, Clone)]
pub struct DerivationContract {
    /// Input file glob or path pattern.
    pub inputs: String,
    /// Output file glob or path pattern.
    pub outputs: String,
    /// Command template to reproduce outputs from inputs.
    pub command: String,
}

/// Figure generation settings (`[figures]`).
#[derive(Debug, Clone, Default)]
pub struct FiguresConfig {
    pub enabled: bool,
    /// Script or binary that renders declared plots.
    pub generator: String,
    pub plots: Vec<FigurePlot>,
}

/// Declared figure plot (`[[figures.plot]]`).
#[derive(Debug, Clone)]
pub struct FigurePlot {
    /// Plot kind (e.g. `scatter`, `histogram`).
    pub plot_type: String,
    /// Glob matching data files to plot.
    pub pattern: String,
    pub x_label: String,
    pub y_label: String,
}

/// Domain audit flags (`[audit]`).
#[derive(Debug, Clone, Default)]
pub struct AuditConfig {
    pub domain: AuditDomainFlags,
    pub validation: AuditValidationFlags,
    pub claims: Vec<ClaimValidator>,
}

/// Domain-scoped audit toggles.
#[derive(Debug, Clone, Default)]
pub struct AuditDomainFlags {
    /// Verify simulation config files match declared parameters.
    pub config_fidelity: bool,
    /// Cross-reference topology atom indices against domain entity groups.
    pub topology_crossref: bool,
    /// Check GROMACS `.mdp` run-parameter headers for consistency.
    pub mdp_headers: bool,
}

/// Validation-scoped audit toggles.
#[derive(Debug, Clone, Default)]
pub struct AuditValidationFlags {
    /// Run scientific-claim validators against module outputs.
    pub scientific_claims: bool,
    /// Verify reported simulation time from config fields.
    pub simulation_time: bool,
}

/// Scientific claim validator (`[[audit.claims.validator]]`).
#[derive(Debug, Clone)]
pub struct ClaimValidator {
    /// Key or glob matching a value in module output JSON.
    pub key_pattern: String,
    pub output_file: String,
    /// Validator algorithm (e.g. `range`, `zones`).
    pub validator_type: String,
    pub zones: Vec<ClaimZone>,
    /// Inclusive min/max when using a simple range validator.
    pub expected_range: Option<(f64, f64)>,
}

/// Named zone for claim validation.
#[derive(Debug, Clone)]
pub struct ClaimZone {
    pub name: String,
    pub min: f64,
    pub max: f64,
}

/// Domain-specific acceptance tolerances (`[tolerances]`).
///
/// Defaults to 2.0 kJ for both tiers (generic RMSD acceptance).
#[derive(Debug, Clone)]
pub struct TolerancesConfig {
    pub tier1_rmsd_kj_max: f64,
    pub tier2_rmsd_kj_max: f64,
}

const DEFAULT_RMSD_KJ: f64 = 2.0;

impl Default for TolerancesConfig {
    fn default() -> Self {
        Self {
            tier1_rmsd_kj_max: DEFAULT_RMSD_KJ,
            tier2_rmsd_kj_max: DEFAULT_RMSD_KJ,
        }
    }
}

/// Simulation time field mapping (`[simulation_time]`).
#[derive(Debug, Clone)]
pub struct SimTimeConfig {
    /// Config file format (e.g. `mdp`, `toml`).
    pub config_format: String,
    /// Field name for integration step count.
    pub nsteps_field: String,
    /// Field name for timestep size.
    pub dt_field: String,
    /// Physical unit of the timestep (e.g. `ps`, `fs`).
    pub time_unit: String,
}

impl Default for SimTimeConfig {
    fn default() -> Self {
        Self {
            config_format: String::new(),
            nsteps_field: "nsteps".to_string(),
            dt_field: "dt".to_string(),
            time_unit: "ps".to_string(),
        }
    }
}

impl DomainProfile {
    /// Load a `domain_profile.toml` from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load(path: &Path) -> Result<Self, crate::SporeError> {
        let content = std::fs::read_to_string(path).map_err(|e| crate::SporeError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;
        Self::parse(&content).map_err(|detail| crate::SporeError::Parse {
            path: path.to_path_buf(),
            detail,
        })
    }

    /// Load from a path, returning `None` if the file is missing or cannot be parsed.
    #[must_use]
    pub fn try_load(path: &Path) -> Option<Self> {
        if !path.exists() {
            return None;
        }
        Self::load(path).ok()
    }

    /// Try to load from a pseudoSpore root directory (looks for `domain_profile.toml`).
    #[must_use]
    pub fn from_spore_root(root: &Path) -> Option<Self> {
        Self::try_load(&root.join("domain_profile.toml"))
    }

    /// Whether index translation is enabled (defaults to `false` when `[translation]` is absent).
    #[must_use]
    pub fn translation_enabled(&self) -> bool {
        self.translation.as_ref().is_some_and(|t| t.enabled)
    }

    /// Whether figure generation is enabled (defaults to `true` when `[figures]` is absent).
    #[must_use]
    pub fn figures_enabled(&self) -> bool {
        self.figures.as_ref().is_none_or(|f| f.enabled)
    }

    /// Entity groups for translation, if configured.
    #[must_use]
    pub fn translation_entity_groups(&self) -> Option<&[EntityGroup]> {
        self.translation
            .as_ref()
            .filter(|t| t.enabled)
            .map(|t| t.entity_groups.as_slice())
    }

    fn parse(content: &str) -> Result<Self, String> {
        let table: toml::Table = content
            .parse()
            .map_err(|e| format!("Failed to parse domain_profile.toml: {e}"))?;

        let profile = table
            .get("profile")
            .and_then(|v| v.as_table())
            .ok_or("Missing [profile] section")?;

        let id = profile
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let version = profile
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();
        let spring = profile
            .get("spring")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let tools = profile
            .get("tools")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let modules = table
            .get("module")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| {
                        let t = m.as_table()?;
                        Some(ProfileModule {
                            name: t.get("name")?.as_str()?.to_string(),
                            description: t
                                .get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            check_command: t
                                .get("check_command")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let check_commands = table
            .get("check")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| {
                        let t = c.as_table()?;
                        Some(CheckCommand {
                            name: t.get("name")?.as_str()?.to_string(),
                            command: t.get("command")?.as_str()?.to_string(),
                            expected_exit: t
                                .get("expected_exit")
                                .and_then(toml::Value::as_integer)
                                .unwrap_or(0) as i32,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let translation = parse::translation(table.get("translation"));
        let derivation = parse::derivation(&table);
        let figures = parse::figures(&table);
        let audit = parse::audit(&table);
        let simulation_time = parse::sim_time(&table);
        let tolerances = parse::tolerances(table.get("tolerances"));

        Ok(Self {
            id,
            version,
            spring,
            tools,
            modules,
            check_commands,
            translation,
            derivation,
            figures,
            audit,
            simulation_time,
            tolerances,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const VALID_PROFILE: &str = r#"
[profile]
id = "test-domain"
version = "1.2.3"
spring = "hotSpring"
tools = ["gromacs", "plumed"]

[[module]]
name = "mod-a"
description = "Module A"
check_command = "true"

[[check]]
name = "smoke"
command = "echo ok"
expected_exit = 0

[translation]
enabled = true
domain_frame = "pdb"
computation_frame = "index"
topology_format = "gro"

[[translation.entity_group]]
name = "backbone"
atoms = ["CA"]
residue_filter = ["ALA"]

[figures]
enabled = false
generator = "python3"

[[figures.plot]]
type = "line"
pattern = "*.dat"
x_label = "t"
y_label = "f"

[audit]
config_fidelity = true
scientific_claims = true

[simulation_time]
config_format = "grompp"
time_unit = "ns"
"#;

    #[test]
    fn load_valid_profile() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("domain_profile.toml");
        fs::write(&path, VALID_PROFILE).expect("write profile");
        let profile = DomainProfile::load(&path).expect("load valid profile");
        assert_eq!(profile.id, "test-domain", "profile id");
        assert_eq!(profile.version, "1.2.3", "profile version");
        assert_eq!(profile.spring.as_deref(), Some("hotSpring"));
        assert_eq!(profile.tools.len(), 2);
        assert_eq!(profile.modules.len(), 1);
        assert_eq!(profile.check_commands.len(), 1);
        assert!(profile.translation_enabled(), "translation.enabled = true");
        assert!(!profile.figures_enabled(), "figures.enabled = false");
    }

    #[test]
    fn load_invalid_toml_fails() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("domain_profile.toml");
        fs::write(&path, "not valid {{{ toml").expect("write bad toml");
        let err = DomainProfile::load(&path).unwrap_err();
        assert!(
            err.to_string().contains("parse"),
            "expected parse error, got: {err}"
        );
    }

    #[test]
    fn load_missing_profile_section_fails() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("domain_profile.toml");
        fs::write(&path, "[other]\nkey = 1").expect("write profile");
        let err = DomainProfile::load(&path).unwrap_err();
        assert!(
            err.to_string().contains("Missing [profile]"),
            "expected missing section error, got: {err}"
        );
    }

    #[test]
    fn try_load_returns_none_for_missing_or_invalid() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(DomainProfile::try_load(&dir.path().join("missing.toml")).is_none());
        let bad = dir.path().join("domain_profile.toml");
        fs::write(&bad, "bad {{{").expect("write");
        assert!(DomainProfile::try_load(&bad).is_none());
        fs::write(&bad, VALID_PROFILE).expect("write valid");
        assert_eq!(
            DomainProfile::from_spore_root(dir.path()).expect("load").id,
            "test-domain"
        );
    }

    fn load_from_content(content: &str) -> DomainProfile {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("domain_profile.toml");
        fs::write(&path, content).expect("write profile");
        DomainProfile::load(&path).expect("load profile")
    }

    #[test]
    fn section_defaults_and_overrides() {
        let minimal = load_from_content("[profile]\nid = \"minimal\"\nversion = \"0.1.0\"\n");
        assert!(minimal.figures_enabled(), "figures default to enabled");
        assert!(
            !minimal.translation_enabled(),
            "translation default to disabled"
        );
        let with_figs = load_from_content(
            "[profile]\nid = \"fig\"\nversion = \"0.1.0\"\n\n[figures]\nenabled = true\n",
        );
        assert!(with_figs.figures_enabled());
    }
}
