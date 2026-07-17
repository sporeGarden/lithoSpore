// SPDX-License-Identifier: AGPL-3.0-or-later

//! TOML section parsers for `domain_profile.toml`.
//!
//! Each function parses one optional section from the profile document.
//! All are pure functions consuming a `toml::Value` reference.

use super::{
    AuditConfig, AuditDomainFlags, AuditValidationFlags, ClaimValidator, ClaimZone,
    DEFAULT_RMSD_KJ, DerivationConfig, DerivationContract, EntityGroup, FigurePlot, FiguresConfig,
    SimTimeConfig, TolerancesConfig, TranslationConfig,
};

pub fn translation(section: Option<&toml::Value>) -> Option<TranslationConfig> {
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

pub fn derivation(table: &toml::Table) -> Option<DerivationConfig> {
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

pub fn figures(table: &toml::Table) -> Option<FiguresConfig> {
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

pub fn audit(table: &toml::Table) -> Option<AuditConfig> {
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

pub fn tolerances(section: Option<&toml::Value>) -> Option<TolerancesConfig> {
    let t = section?.as_table()?;
    let get = |key| {
        t.get(key)
            .and_then(toml::Value::as_float)
            .unwrap_or(DEFAULT_RMSD_KJ)
    };
    Some(TolerancesConfig {
        tier1_rmsd_kj_max: get("tier1_rmsd_kj_max"),
        tier2_rmsd_kj_max: get("tier2_rmsd_kj_max"),
    })
}

pub fn sim_time(table: &toml::Table) -> Option<SimTimeConfig> {
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

    #[test]
    fn translation_none_on_missing() {
        assert!(translation(None).is_none());
    }

    #[test]
    fn translation_parses_enabled() {
        let toml_str = r#"
enabled = true
domain_frame = "pdb"
computation_frame = "index"
topology_format = "gro"
"#;
        let val: toml::Value = toml_str.parse().unwrap();
        let t = translation(Some(&val)).unwrap();
        assert!(t.enabled);
        assert_eq!(t.domain_frame, "pdb");
        assert_eq!(t.computation_frame, "index");
    }

    #[test]
    fn tolerances_none_on_missing() {
        assert!(tolerances(None).is_none());
    }

    #[test]
    fn tolerances_uses_defaults_and_overrides() {
        let toml_str = "tier1_rmsd_kj_max = 3.5";
        let val: toml::Value = toml_str.parse().unwrap();
        let t = tolerances(Some(&val)).unwrap();
        assert!((t.tier1_rmsd_kj_max - 3.5).abs() < f64::EPSILON);
        assert!((t.tier2_rmsd_kj_max - DEFAULT_RMSD_KJ).abs() < f64::EPSILON);
    }

    #[test]
    fn derivation_parses_contracts() {
        let toml_str = r#"
[derivation]
tool = "plumed"
find_paths = ["*.dat"]
[[derivation.contract]]
inputs = "data/{module}/HILLS"
outputs = "outputs/{module}/fes.dat"
command = "plumed sum_hills --hills {input} --outfile {output}"
"#;
        let table: toml::Table = toml_str.parse().unwrap();
        let d = derivation(&table).unwrap();
        assert_eq!(d.tool, "plumed");
        assert_eq!(d.contracts.len(), 1);
        assert!(d.contracts[0].command.contains("sum_hills"));
    }

    #[test]
    fn figures_defaults_generator() {
        let toml_str = "[figures]\nenabled = true\n";
        let table: toml::Table = toml_str.parse().unwrap();
        let f = figures(&table).unwrap();
        assert!(f.enabled);
        assert_eq!(f.generator, "python3");
    }

    #[test]
    fn audit_parses_flags() {
        let toml_str = "[audit]\nconfig_fidelity = true\nscientific_claims = true\n";
        let table: toml::Table = toml_str.parse().unwrap();
        let a = audit(&table).unwrap();
        assert!(a.domain.config_fidelity);
        assert!(a.validation.scientific_claims);
    }

    #[test]
    fn sim_time_defaults() {
        let toml_str = "[simulation_time]\nconfig_format = \"mdp\"\n";
        let table: toml::Table = toml_str.parse().unwrap();
        let s = sim_time(&table).unwrap();
        assert_eq!(s.config_format, "mdp");
        assert_eq!(s.nsteps_field, "nsteps");
        assert_eq!(s.dt_field, "dt");
        assert_eq!(s.time_unit, "ps");
    }
}
