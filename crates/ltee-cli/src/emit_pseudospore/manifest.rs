// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

/// Generate data.toml — guideStone data manifest with per-dataset BLAKE3 hashes.
pub(super) fn generate_data_manifest(
    data_root: &Path,
    name: &str,
    version: &str,
    spring_name: &str,
) -> String {
    let mut output = String::new();
    output.push_str("# Data Manifest — guideStone data component\n");
    output.push_str("# Per wateringHole/TARGETED_GUIDESTONE_STANDARD v1.0\n");
    writeln!(output, "# Artifact: {name} v{version}\n").unwrap();
    output.push_str("[manifest]\n");
    output.push_str("standard = \"wateringHole/TARGETED_GUIDESTONE_STANDARD v1.0\"\n");
    output.push_str("hash_method = \"blake3\"\n");
    output.push_str(
        "directory_hash = \"blake3(concat(blake3(file) for file in sorted(walk(dir))))\"\n\n",
    );

    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();

    // Scan data/ for datasets (each subdirectory = one dataset, plus root-level files)
    let mut entries: Vec<PathBuf> = Vec::new();
    if let Ok(dir) = fs::read_dir(data_root) {
        for entry in dir.flatten() {
            entries.push(entry.path());
        }
    }
    entries.sort();

    for entry in &entries {
        let rel_name = entry
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let local_path = format!("data/{rel_name}");

        if entry.is_dir() {
            // Directory dataset: compute combined hash of all files
            let mut hasher_input = String::new();
            collect_file_hashes(entry, &mut hasher_input);
            let hash = blake3_string(&hasher_input);

            let id = rel_name.clone();
            output.push_str("[[dataset]]\n");
            writeln!(output, "id = \"{id}\"").unwrap();
            let spring_lower = spring_name.to_lowercase();
            writeln!(output, "source_uri = \"urn:{spring_lower}:{id}\"").unwrap();
            output.push_str("license = \"AGPL-3.0-or-later\"\n");
            writeln!(output, "local_path = \"{local_path}/\"").unwrap();
            writeln!(output, "blake3 = \"{hash}\"").unwrap();
            writeln!(output, "retrieved = \"{date}\"").unwrap();
            writeln!(
                output,
                "refresh_command = \"# Re-run simulation; see configs/{id}/\""
            )
            .unwrap();
            writeln!(output, "upstream_spring = \"{spring_name}\"").unwrap();
            writeln!(
                output,
                "upstream_braid = \"urn:provenance:braid:{spring_lower}-v{version}\"\n"
            )
            .unwrap();
        } else {
            // Single file dataset (e.g. 2D24.pdb)
            let hash = blake3_file(entry);
            let id = rel_name.replace('.', "-");
            let is_pdb = Path::new(&rel_name)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("pdb"));

            output.push_str("[[dataset]]\n");
            writeln!(output, "id = \"{id}\"").unwrap();
            if is_pdb {
                let pdb_id = Path::new(&rel_name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&id);
                writeln!(
                    output,
                    "source_uri = \"https://www.rcsb.org/structure/{pdb_id}\""
                )
                .unwrap();
                output.push_str("license = \"CC0\"\n");
                writeln!(
                    output,
                    "refresh_command = \"curl -sL https://files.rcsb.org/download/{rel_name} -o {local_path}\""
                )
                .unwrap();
            } else {
                let spring_lower = spring_name.to_lowercase();
                writeln!(output, "source_uri = \"urn:{spring_lower}:{id}\"").unwrap();
                output.push_str("license = \"AGPL-3.0-or-later\"\n");
                output.push_str("refresh_command = \"# Manual: re-obtain from source\"\n");
            }
            writeln!(output, "local_path = \"{local_path}\"").unwrap();
            writeln!(output, "blake3 = \"{hash}\"").unwrap();
            writeln!(output, "retrieved = \"{date}\"").unwrap();
            writeln!(output, "upstream_spring = \"{spring_name}\"\n").unwrap();
        }
    }

    output
}

fn collect_file_hashes(dir: &Path, output: &mut String) {
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            paths.push(entry.path());
        }
    }
    paths.sort();
    for path in paths {
        if path.is_file() {
            let h = blake3_file(&path);
            output.push_str(&h);
        } else if path.is_dir() {
            collect_file_hashes(&path, output);
        }
    }
}

fn blake3_file(path: &Path) -> String {
    let data = fs::read(path).unwrap_or_default();
    let hash = blake3::hash(&data);
    hash.to_hex().to_string()
}

fn blake3_string(input: &str) -> String {
    let hash = blake3::hash(input.as_bytes());
    hash.to_hex().to_string()
}

/// Generate tolerances.toml with named tolerances and scientific justification.
pub(super) fn generate_tolerances_justified(
    profile: Option<&pseudospore_core::DomainProfile>,
) -> String {
    let mut output = String::new();
    output.push_str("# Named Tolerances — guideStone validation contract\n");
    output.push_str("# Per wateringHole/TARGETED_GUIDESTONE_STANDARD v1.0\n");
    if let Some(p) = profile {
        writeln!(output, "# Domain profile: {} v{}", p.id, p.version).unwrap();
    }
    output.push('\n');

    // Universal tolerance: BLAKE3 checksum integrity (always present)
    output.push_str("[[tolerance]]\n");
    output.push_str("name = \"checksum_integrity\"\n");
    output.push_str("value = 0\n");
    output.push_str("unit = \"bits\"\n");
    output.push_str("justification = \"BLAKE3 cryptographic hash; any bit flip is a failure\"\n\n");

    // Domain-specific tolerances: populated from profile or manually post-emission.
    // The profile declares check_commands; tolerances should match those checks.
    if let Some(p) = profile {
        writeln!(
            output,
            "# Add domain-specific tolerances for {} here.",
            p.id
        )
        .unwrap();
        output.push_str("# Each tolerance should have a physical or mathematical derivation.\n\n");
    }

    output
}

/// Generate a stub `threshold_calibration.toml` per `DERIVATION_ANCHORING_STANDARD`.
pub(super) fn generate_calibration_stub(
    profile: Option<&pseudospore_core::DomainProfile>,
) -> String {
    use std::fmt::Write as _;

    let mut output = String::new();
    output.push_str("# DERIVATION ANCHORING — threshold_calibration.toml\n");
    output.push_str("# See DERIVATION_ANCHORING_STANDARD.md for the 5-layer chain.\n");
    output.push_str("# Populate each [[constant]] with empirical derivation data.\n\n");
    output.push_str("[metadata]\n");
    output.push_str("standard = \"DERIVATION_ANCHORING_STANDARD v1.0\"\n");
    if let Some(p) = profile {
        writeln!(output, "domain = \"{}\"", p.id).unwrap();
    } else {
        output.push_str("domain = \"unknown\"\n");
    }
    output.push_str("status = \"STUB\"\n\n");
    output.push_str("# Example constant (replace with actual calibration):\n");
    output.push_str("# [[constant]]\n");
    output.push_str("# name = \"rmsd_convergence_threshold\"\n");
    output.push_str("# value = 2.0\n");
    output.push_str("# unit = \"kJ/mol\"\n");
    output.push_str("# layer1_source = \"paper Table N\"\n");
    output.push_str("# layer2_method = \"block-averaged standard error\"\n");
    output.push_str("# layer3_calibration = \"3x SEM across N trajectories\"\n");
    output.push_str("# layer4_validation = \"Phase 0 self-consistency\"\n");
    output.push_str("# layer5_runtime = \"litho validate --tier 0\"\n");
    output.push_str("# _anchoring = \"CALIBRATED\"\n");
    output
}
