// SPDX-License-Identifier: AGPL-3.0-or-later

//! Domain profile — declarative configuration for domain-specific pseudoSpore behavior.
//!
//! A `domain_profile.toml` at the pseudoSpore root tells emit/audit/promote what
//! domain-specific logic to apply. When absent, only core (domain-agnostic) checks run.
//!
//! The profile is intentionally generic: it declares WHAT to check, not HOW.
//! Domain-specific implementations live in their respective springs.

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
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        Self::parse(&content)
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

        let translation = parse_translation(table.get("translation"));
        let derivation = parse_derivation(&table);
        let figures = parse_figures(&table);
        let audit = parse_audit(&table);
        let simulation_time = parse_sim_time(&table);

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
        })
    }
}

fn parse_translation(section: Option<&toml::Value>) -> Option<TranslationConfig> {
    let table = section?.as_table()?;
    let enabled = table
        .get("enabled")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    let domain_frame = table
        .get("domain_frame")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let computation_frame = table
        .get("computation_frame")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let topology_format = table
        .get("topology_format")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let mut entity_groups = Vec::new();
    if let Some(groups) = table.get("entity_group").and_then(|v| v.as_array()) {
        for group in groups {
            if let Some(g) = group.as_table() {
                entity_groups.push(EntityGroup {
                    name: g
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    atoms: g
                        .get("atoms")
                        .and_then(|v| v.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|s| s.as_str().map(str::to_string))
                                .collect()
                        })
                        .unwrap_or_default(),
                    residue_filter: g
                        .get("residue_filter")
                        .and_then(|v| v.as_array())
                        .map(|a| {
                            a.iter()
                                .filter_map(|s| s.as_str().map(str::to_string))
                                .collect()
                        })
                        .unwrap_or_default(),
                });
            }
        }
    }

    Some(TranslationConfig {
        enabled,
        domain_frame,
        computation_frame,
        topology_format,
        entity_groups,
    })
}

fn parse_derivation(table: &toml::Table) -> Option<DerivationConfig> {
    let section = table.get("derivation")?.as_table()?;
    let tool = section
        .get("tool")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let find_paths = section
        .get("find_paths")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let mut contracts = Vec::new();
    if let Some(arr) = section.get("contract").and_then(|v| v.as_array()) {
        for c in arr {
            if let Some(ct) = c.as_table() {
                contracts.push(DerivationContract {
                    inputs: ct
                        .get("inputs")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    outputs: ct
                        .get("outputs")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    command: ct
                        .get("command")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                });
            }
        }
    }

    Some(DerivationConfig {
        tool,
        find_paths,
        contracts,
    })
}

fn parse_figures(table: &toml::Table) -> Option<FiguresConfig> {
    let section = table.get("figures")?.as_table()?;
    let enabled = section
        .get("enabled")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    let generator = section
        .get("generator")
        .and_then(|v| v.as_str())
        .unwrap_or("python3")
        .to_string();

    let mut plots = Vec::new();
    if let Some(arr) = section.get("plot").and_then(|v| v.as_array()) {
        for p in arr {
            if let Some(pt) = p.as_table() {
                plots.push(FigurePlot {
                    plot_type: pt
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    pattern: pt
                        .get("pattern")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    x_label: pt
                        .get("x_label")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    y_label: pt
                        .get("y_label")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                });
            }
        }
    }

    Some(FiguresConfig {
        enabled,
        generator,
        plots,
    })
}

fn parse_audit(table: &toml::Table) -> Option<AuditConfig> {
    let section = table.get("audit")?.as_table()?;
    let config_fidelity = section
        .get("config_fidelity")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    let scientific_claims = section
        .get("scientific_claims")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    let simulation_time = section
        .get("simulation_time")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    let topology_crossref = section
        .get("topology_crossref")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);
    let mdp_headers = section
        .get("mdp_headers")
        .and_then(toml::Value::as_bool)
        .unwrap_or(false);

    let mut claims = Vec::new();
    if let Some(claims_section) = section.get("claims").and_then(|v| v.as_table())
        && let Some(validators) = claims_section.get("validator").and_then(|v| v.as_array())
    {
        for v in validators {
            if let Some(vt) = v.as_table() {
                let mut zones = Vec::new();
                if let Some(z_arr) = vt.get("zones").and_then(|v| v.as_array()) {
                    for z in z_arr {
                        if let Some(zt) = z.as_table() {
                            zones.push(ClaimZone {
                                name: zt
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                min: zt.get("min").and_then(toml::Value::as_float).unwrap_or(0.0),
                                max: zt.get("max").and_then(toml::Value::as_float).unwrap_or(0.0),
                            });
                        }
                    }
                }

                let expected_range = vt
                    .get("expected_range")
                    .and_then(|v| v.as_array())
                    .and_then(|a| {
                        if a.len() == 2 {
                            Some((a[0].as_float()?, a[1].as_float()?))
                        } else {
                            None
                        }
                    });

                claims.push(ClaimValidator {
                    key_pattern: vt
                        .get("key_pattern")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    output_file: vt
                        .get("output_file")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    validator_type: vt
                        .get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    zones,
                    expected_range,
                });
            }
        }
    }

    Some(AuditConfig {
        domain: AuditDomainFlags {
            config_fidelity,
            topology_crossref,
            mdp_headers,
        },
        validation: AuditValidationFlags {
            scientific_claims,
            simulation_time,
        },
        claims,
    })
}

fn parse_sim_time(table: &toml::Table) -> Option<SimTimeConfig> {
    let section = table.get("simulation_time")?.as_table()?;
    let config_format = section
        .get("config_format")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let (nsteps_field, dt_field) =
        if let Some(fields) = section.get("time_fields").and_then(|v| v.as_table()) {
            (
                fields
                    .get("nsteps")
                    .and_then(|v| v.as_str())
                    .unwrap_or("nsteps")
                    .to_string(),
                fields
                    .get("dt")
                    .and_then(|v| v.as_str())
                    .unwrap_or("dt")
                    .to_string(),
            )
        } else {
            ("nsteps".to_string(), "dt".to_string())
        };

    let time_unit = section
        .get("time_unit")
        .and_then(|v| v.as_str())
        .unwrap_or("ps")
        .to_string();

    Some(SimTimeConfig {
        config_format,
        nsteps_field,
        dt_field,
        time_unit,
    })
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
            err.contains("Failed to parse"),
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
            err.contains("Missing [profile]"),
            "expected missing section error, got: {err}"
        );
    }

    #[test]
    fn try_load_missing_file_returns_none() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("missing.toml");
        assert!(DomainProfile::try_load(&path).is_none());
    }

    #[test]
    fn try_load_invalid_file_returns_none() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("domain_profile.toml");
        fs::write(&path, "bad {{{").expect("write");
        assert!(DomainProfile::try_load(&path).is_none());
    }

    #[test]
    fn from_spore_root_loads_profile() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("domain_profile.toml"), VALID_PROFILE).expect("write");
        let profile = DomainProfile::from_spore_root(dir.path()).expect("from_spore_root");
        assert_eq!(profile.id, "test-domain");
    }

    fn load_from_content(content: &str) -> DomainProfile {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("domain_profile.toml");
        fs::write(&path, content).expect("write profile");
        DomainProfile::load(&path).expect("load profile")
    }

    #[test]
    fn figures_enabled_defaults_true_when_section_absent() {
        let profile = load_from_content(
            r#"
[profile]
id = "minimal"
version = "0.1.0"
"#,
        );
        assert!(
            profile.figures_enabled(),
            "figures should default to enabled when [figures] absent"
        );
    }

    #[test]
    fn translation_enabled_defaults_false_when_section_absent() {
        let profile = load_from_content(
            r#"
[profile]
id = "minimal"
version = "0.1.0"
"#,
        );
        assert!(
            !profile.translation_enabled(),
            "translation should default to disabled when [translation] absent"
        );
    }

    #[test]
    fn figures_enabled_true_when_section_enabled() {
        let profile = load_from_content(
            r#"
[profile]
id = "fig"
version = "0.1.0"

[figures]
enabled = true
"#,
        );
        assert!(profile.figures_enabled());
    }
}
