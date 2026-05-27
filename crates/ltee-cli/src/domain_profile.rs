// SPDX-License-Identifier: AGPL-3.0-or-later

//! Domain Profile — declarative science-domain configuration for pseudoSpore tooling.
//!
//! A `domain_profile.toml` at the pseudoSpore root tells emit/audit/promote what
//! domain-specific logic to apply. When absent, only core (domain-agnostic) checks run.
//! When present, the profile drives figure generation, index translation, derivation
//! verification, and scientific claim validation.

use std::fs;
use std::path::Path;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DomainProfile {
    pub id: String,
    pub version: String,
    pub tools: Vec<String>,
    pub translation: TranslationConfig,
    pub derivation: DerivationConfig,
    pub figures: FiguresConfig,
    pub audit: AuditConfig,
    pub simulation_time: SimTimeConfig,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct TranslationConfig {
    pub enabled: bool,
    pub domain_frame: String,
    pub computation_frame: String,
    pub topology_format: String,
    pub entity_groups: Vec<EntityGroup>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EntityGroup {
    pub name: String,
    pub atoms: Vec<String>,
    pub residue_filter: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DerivationConfig {
    pub tool: String,
    pub find_paths: Vec<String>,
    pub contracts: Vec<DerivationContract>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DerivationContract {
    pub inputs: String,
    pub outputs: String,
    pub command: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FiguresConfig {
    pub enabled: bool,
    pub generator: String,
    pub plots: Vec<FigurePlot>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FigurePlot {
    pub plot_type: String,
    pub pattern: String,
    pub x_label: String,
    pub y_label: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct AuditConfig {
    pub config_fidelity: bool,
    pub scientific_claims: bool,
    pub simulation_time: bool,
    pub topology_crossref: bool,
    pub mdp_headers: bool,
    pub claims: Vec<ClaimValidator>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClaimValidator {
    pub key_pattern: String,
    pub output_file: String,
    pub validator_type: String,
    pub zones: Vec<ClaimZone>,
    pub expected_range: Option<(f64, f64)>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClaimZone {
    pub name: String,
    pub min: f64,
    pub max: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SimTimeConfig {
    pub config_format: String,
    pub nsteps_field: String,
    pub dt_field: String,
    pub time_unit: String,
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            domain_frame: String::new(),
            computation_frame: String::new(),
            topology_format: String::new(),
            entity_groups: Vec::new(),
        }
    }
}

impl Default for DerivationConfig {
    fn default() -> Self {
        Self {
            tool: String::new(),
            find_paths: Vec::new(),
            contracts: Vec::new(),
        }
    }
}

impl Default for FiguresConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            generator: String::new(),
            plots: Vec::new(),
        }
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            config_fidelity: false,
            scientific_claims: false,
            simulation_time: false,
            topology_crossref: false,
            mdp_headers: false,
            claims: Vec::new(),
        }
    }
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

/// Load a domain profile from a specific file path.
/// Returns None if the file doesn't exist or fails to parse.
pub fn load_from_file(path: &Path) -> Option<DomainProfile> {
    if !path.exists() {
        return None;
    }
    let content = fs::read_to_string(path).ok()?;
    parse_profile_content(&content)
}

/// Load a domain profile from `domain_profile.toml` at the given root.
/// Returns None if the file doesn't exist (graceful degradation).
pub fn load_domain_profile(root: &Path) -> Option<DomainProfile> {
    let path = root.join("domain_profile.toml");
    if !path.exists() {
        return None;
    }

    let content = fs::read_to_string(&path).ok()?;
    parse_profile_content(&content)
}

fn parse_profile_content(content: &str) -> Option<DomainProfile> {
    let table: toml::Table = content.parse().ok()?;

    let profile_section = table.get("profile")?.as_table()?;
    let id = profile_section.get("id")?.as_str()?.to_string();
    let version = profile_section.get("version").and_then(|v| v.as_str()).unwrap_or("1.0").to_string();
    let tools = profile_section.get("tools")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|s| s.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let translation = parse_translation(table.get("translation"));
    let derivation = parse_derivation(&table);
    let figures = parse_figures(&table);
    let audit = parse_audit(&table);
    let simulation_time = parse_sim_time(&table);

    Some(DomainProfile {
        id,
        version,
        tools,
        translation,
        derivation,
        figures,
        audit,
        simulation_time,
    })
}

fn parse_translation(section: Option<&toml::Value>) -> TranslationConfig {
    let Some(table) = section.and_then(|v| v.as_table()) else {
        return TranslationConfig::default();
    };

    let enabled = table.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let domain_frame = table.get("domain_frame").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let computation_frame = table.get("computation_frame").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let topology_format = table.get("topology_format").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let mut entity_groups = Vec::new();
    if let Some(groups) = table.get("entity_group").and_then(|v| v.as_array()) {
        for group in groups {
            if let Some(g) = group.as_table() {
                let name = g.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let atoms = g.get("atoms").and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|s| s.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                let residue_filter = g.get("residue_filter").and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|s| s.as_str().map(|s| s.to_string())).collect())
                    .unwrap_or_default();
                entity_groups.push(EntityGroup { name, atoms, residue_filter });
            }
        }
    }

    TranslationConfig { enabled, domain_frame, computation_frame, topology_format, entity_groups }
}

fn parse_derivation(table: &toml::Table) -> DerivationConfig {
    let Some(section) = table.get("derivation").and_then(|v| v.as_table()) else {
        return DerivationConfig::default();
    };

    let tool = section.get("tool").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let find_paths = section.get("find_paths").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|s| s.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let mut contracts = Vec::new();
    if let Some(arr) = section.get("contract").and_then(|v| v.as_array()) {
        for c in arr {
            if let Some(ct) = c.as_table() {
                contracts.push(DerivationContract {
                    inputs: ct.get("inputs").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    outputs: ct.get("outputs").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    command: ct.get("command").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                });
            }
        }
    }

    DerivationConfig { tool, find_paths, contracts }
}

fn parse_figures(table: &toml::Table) -> FiguresConfig {
    let Some(section) = table.get("figures").and_then(|v| v.as_table()) else {
        return FiguresConfig::default();
    };

    let enabled = section.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let generator = section.get("generator").and_then(|v| v.as_str()).unwrap_or("python3").to_string();

    let mut plots = Vec::new();
    if let Some(arr) = section.get("plot").and_then(|v| v.as_array()) {
        for p in arr {
            if let Some(pt) = p.as_table() {
                plots.push(FigurePlot {
                    plot_type: pt.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    pattern: pt.get("pattern").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    x_label: pt.get("x_label").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    y_label: pt.get("y_label").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                });
            }
        }
    }

    FiguresConfig { enabled, generator, plots }
}

fn parse_audit(table: &toml::Table) -> AuditConfig {
    let Some(section) = table.get("audit").and_then(|v| v.as_table()) else {
        return AuditConfig::default();
    };

    let config_fidelity = section.get("config_fidelity").and_then(|v| v.as_bool()).unwrap_or(false);
    let scientific_claims = section.get("scientific_claims").and_then(|v| v.as_bool()).unwrap_or(false);
    let simulation_time = section.get("simulation_time").and_then(|v| v.as_bool()).unwrap_or(false);
    let topology_crossref = section.get("topology_crossref").and_then(|v| v.as_bool()).unwrap_or(false);
    let mdp_headers = section.get("mdp_headers").and_then(|v| v.as_bool()).unwrap_or(false);

    let mut claims = Vec::new();
    if let Some(claims_section) = section.get("claims").and_then(|v| v.as_table()) {
        if let Some(validators) = claims_section.get("validator").and_then(|v| v.as_array()) {
            for v in validators {
                if let Some(vt) = v.as_table() {
                    let mut zones = Vec::new();
                    if let Some(z_arr) = vt.get("zones").and_then(|v| v.as_array()) {
                        for z in z_arr {
                            if let Some(zt) = z.as_table() {
                                zones.push(ClaimZone {
                                    name: zt.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                    min: zt.get("min").and_then(|v| v.as_float()).unwrap_or(0.0),
                                    max: zt.get("max").and_then(|v| v.as_float()).unwrap_or(0.0),
                                });
                            }
                        }
                    }

                    let expected_range = vt.get("expected_range").and_then(|v| v.as_array()).and_then(|a| {
                        if a.len() == 2 {
                            Some((a[0].as_float()?, a[1].as_float()?))
                        } else {
                            None
                        }
                    });

                    claims.push(ClaimValidator {
                        key_pattern: vt.get("key_pattern").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        output_file: vt.get("output_file").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        validator_type: vt.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        zones,
                        expected_range,
                    });
                }
            }
        }
    }

    AuditConfig { config_fidelity, scientific_claims, simulation_time, topology_crossref, mdp_headers, claims }
}

fn parse_sim_time(table: &toml::Table) -> SimTimeConfig {
    let Some(section) = table.get("simulation_time").and_then(|v| v.as_table()) else {
        return SimTimeConfig::default();
    };

    let config_format = section.get("config_format").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let (nsteps_field, dt_field) = if let Some(fields) = section.get("time_fields").and_then(|v| v.as_table()) {
        (
            fields.get("nsteps").and_then(|v| v.as_str()).unwrap_or("nsteps").to_string(),
            fields.get("dt").and_then(|v| v.as_str()).unwrap_or("dt").to_string(),
        )
    } else {
        ("nsteps".to_string(), "dt".to_string())
    };

    let time_unit = section.get("time_unit").and_then(|v| v.as_str()).unwrap_or("ps").to_string();

    SimTimeConfig { config_format, nsteps_field, dt_field, time_unit }
}
