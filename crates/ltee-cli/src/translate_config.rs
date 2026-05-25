// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho translate-config` — generate configs in either domain or computation frame.
//!
//! Reads `index_map.toml` and a template config (e.g. plumed.dat), then emits
//! a version with indices translated to the requested frame. This eliminates
//! manual reindexing and ensures domain experts see PDB-standard numbering
//! while compute configs use runtime-correct indices.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct AtomMapping {
    name: String,
    domain: u64,
    computation: u64,
}

#[derive(Debug)]
#[allow(dead_code)]
struct SystemMap {
    name: String,
    description: String,
    atoms: Vec<AtomMapping>,
}

pub fn run(index_map_path: &str, config_path: &str, frame: &str, output: Option<&str>) {
    let map_path = Path::new(index_map_path);
    let cfg_path = Path::new(config_path);

    if !map_path.exists() {
        eprintln!("ERROR: index_map.toml not found at: {}", map_path.display());
        std::process::exit(1);
    }
    if !cfg_path.exists() {
        eprintln!("ERROR: config file not found at: {}", cfg_path.display());
        std::process::exit(1);
    }

    let map_content = fs::read_to_string(map_path).expect("Failed to read index_map.toml");
    let config_content = fs::read_to_string(cfg_path).expect("Failed to read config file");

    let systems = parse_index_map(&map_content);

    let (from_frame, to_frame) = match frame {
        "domain" => ("computation", "domain"),
        "computation" => ("domain", "computation"),
        _ => {
            eprintln!("ERROR: --frame must be 'domain' or 'computation'");
            std::process::exit(1);
        }
    };

    println!("=== litho translate-config ===");
    println!("  index_map: {}", map_path.display());
    println!("  config:    {}", cfg_path.display());
    println!("  target:    {} frame", frame);
    println!();

    let translated = translate(&config_content, &systems, from_frame, to_frame);

    if let Some(out_path) = output {
        fs::write(out_path, &translated).expect("Failed to write output");
        println!("  Written to: {out_path}");
    } else {
        print!("{translated}");
    }
}

fn parse_index_map(content: &str) -> Vec<SystemMap> {
    let table: toml::Table = content.parse().expect("Failed to parse index_map.toml");
    let mut systems = Vec::new();

    if let Some(toml::Value::Table(systems_table)) = table.get("systems") {
        for (sys_name, sys_val) in systems_table {
            if let toml::Value::Table(sys) = sys_val {
                let description = sys
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let mut atoms = Vec::new();
                if let Some(toml::Value::Table(ring)) = sys.get("ring") {
                    for (atom_name, atom_val) in ring {
                        if atom_name.starts_with('_') {
                            continue;
                        }
                        if let toml::Value::Table(mapping) = atom_val {
                            let domain = mapping
                                .get("domain")
                                .and_then(|v| v.as_integer())
                                .unwrap_or(0) as u64;
                            let computation = mapping
                                .get("computation")
                                .and_then(|v| v.as_integer())
                                .unwrap_or(0) as u64;
                            atoms.push(AtomMapping {
                                name: atom_name.clone(),
                                domain,
                                computation,
                            });
                        }
                    }
                }

                systems.push(SystemMap {
                    name: sys_name.clone(),
                    description,
                    atoms,
                });
            }
        }
    }

    systems
}

fn translate(config: &str, systems: &[SystemMap], from_frame: &str, to_frame: &str) -> String {
    let mut lookup: HashMap<u64, u64> = HashMap::new();
    let mut reverse_names: HashMap<u64, String> = HashMap::new();

    for sys in systems {
        for atom in &sys.atoms {
            let (from_val, to_val) = if from_frame == "computation" {
                (atom.computation, atom.domain)
            } else {
                (atom.domain, atom.computation)
            };
            lookup.insert(from_val, to_val);
            reverse_names.insert(from_val, atom.name.clone());
        }
    }

    let mut result = String::new();
    for line in config.lines() {
        if line.trim_start().starts_with('#') || line.trim().is_empty() {
            result.push_str(line);
            result.push('\n');
            continue;
        }

        // Look for PUCKERING ATOMS= or similar index-bearing directives
        if let Some(atoms_pos) = line.find("ATOMS=") {
            let prefix = &line[..atoms_pos + 6];
            let rest = &line[atoms_pos + 6..];

            // Find end of atom list (next space or end of line)
            let end = rest.find(' ').unwrap_or(rest.len());
            let atom_str = &rest[..end];
            let suffix = &rest[end..];

            let translated_atoms: Vec<String> = atom_str
                .split(',')
                .map(|idx_str| {
                    if let Ok(idx) = idx_str.trim().parse::<u64>() {
                        if let Some(&mapped) = lookup.get(&idx) {
                            mapped.to_string()
                        } else {
                            idx_str.to_string()
                        }
                    } else {
                        idx_str.to_string()
                    }
                })
                .collect();

            result.push_str(prefix);
            result.push_str(&translated_atoms.join(","));
            result.push_str(suffix);
            result.push('\n');
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    // Add frame annotation header
    let mut header = format!(
        "# AUTO-TRANSLATED to {} frame by `litho translate-config`\n",
        to_frame
    );
    header.push_str(&format!(
        "# Source indices were in {} frame\n#\n",
        from_frame
    ));
    header.push_str(&result);
    header
}
