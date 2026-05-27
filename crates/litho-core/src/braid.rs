// SPDX-License-Identifier: AGPL-3.0-or-later

//! Upstream ferment transcript braid ingestion.
//!
//! Braids are small JSON artifacts produced by upstream springs (e.g. wetSpring)
//! that record computation provenance without shipping raw data. The guideStone
//! validates braid metadata (accession, tool, substrate) to prove upstream
//! computation happened, without needing the raw FASTQs or BAMs.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// A ferment transcript braid from an upstream spring.
///
/// Supports two wire formats:
/// - **Sovereign**: full provenance with `dataset_id`, `braid_id`, `computation` block
/// - **Baseline**: flat breseq output with `dataset`, `clones_processed`, `total_mutations`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FermentBraid {
    #[serde(alias = "dataset")]
    #[serde(default)]
    pub dataset_id: String,
    #[serde(default)]
    pub spring: String,
    #[serde(default)]
    pub spring_version: String,
    #[serde(default)]
    pub braid_id: String,
    #[serde(default)]
    pub dag_session_id: String,
    #[serde(default)]
    pub dag_merkle_root: String,
    #[serde(default)]
    pub spine_id: String,
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub computation: Option<BraidComputation>,

    // breseq-style flat braids (no computation block)
    #[serde(default)]
    pub clones_processed: Option<u32>,
    #[serde(default)]
    pub total_mutations: Option<u64>,
    #[serde(default)]
    pub paper: Option<String>,
    #[serde(default)]
    pub reference: Option<String>,
    #[serde(default)]
    pub reference_length_bp: Option<u64>,
    #[serde(default)]
    pub mutation_counts: Option<Vec<CloneMutationCount>>,
}

/// Per-clone mutation count in a breseq baseline braid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneMutationCount {
    pub clone: String,
    pub mutations: u64,
}

/// Computation metadata inside a sovereign pipeline braid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraidComputation {
    pub tool: String,
    #[serde(default)]
    pub tool_version: String,
    #[serde(default)]
    pub substrate: String,
    #[serde(default)]
    pub pipeline: String,
    #[serde(default)]
    pub input_accession: String,
    #[serde(default)]
    pub input_blake3: String,
    #[serde(default)]
    pub output_blake3: String,
    #[serde(default)]
    pub wall_time_seconds: Option<u64>,
    #[serde(default)]
    pub node_count: Option<u32>,
    #[serde(default)]
    pub sovereign_variants: Option<u64>,
    #[serde(default)]
    pub breseq_variants: Option<u64>,
    #[serde(default)]
    pub position_matches: Option<u64>,
}

/// Result of validating a single braid against expected accessions.
#[derive(Debug, Clone)]
pub struct BraidCheck {
    pub braid_id: String,
    pub spring: String,
    pub tool: String,
    pub substrate: String,
    pub accession_ok: bool,
    pub expected_accession: String,
    pub found_accession: String,
    pub file: String,
}

/// Load all braids from a directory, returning them with their source filenames.
#[must_use]
pub fn load_braids(braids_dir: &Path) -> Vec<(String, FermentBraid)> {
    let mut braids = Vec::new();
    let entries = match std::fs::read_dir(braids_dir) {
        Ok(e) => e,
        Err(_) => return braids,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        match std::fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<FermentBraid>(&content) {
                Ok(mut braid) => {
                    // Auto-fill braid_id for baseline braids that lack one
                    if braid.braid_id.is_empty() {
                        let stem = filename.trim_end_matches(".json");
                        braid.braid_id = format!("braid-{stem}");
                    }
                    if braid.spring.is_empty() {
                        braid.spring = "wetSpring".to_string();
                    }
                    braids.push((filename, braid));
                }
                Err(e) => eprintln!("  WARN: could not parse braid {filename}: {e}"),
            },
            Err(e) => eprintln!("  WARN: could not read braid {filename}: {e}"),
        }
    }

    braids.sort_by(|a, b| a.0.cmp(&b.0));
    braids
}

/// Validate braids against expected accessions (from data.toml SRA entries).
/// Returns a check result for each braid that has computation metadata.
#[must_use]
pub fn validate_braids(
    braids: &[(String, FermentBraid)],
    expected_accessions: &[(&str, &str)],
) -> Vec<BraidCheck> {
    let mut checks = Vec::new();

    for (filename, braid) in braids {
        let (tool, substrate, found_accession) = if let Some(ref comp) = braid.computation {
            (
                comp.tool.clone(),
                comp.substrate.clone(),
                comp.input_accession.clone(),
            )
        } else {
            let tool = if braid.total_mutations.is_some() {
                "breseq (baseline)"
            } else {
                "unknown"
            };
            (tool.to_string(), "CPU".to_string(), String::new())
        };

        let (accession_ok, expected) = if found_accession.is_empty() {
            (true, String::new())
        } else {
            match expected_accessions
                .iter()
                .find(|(ds, _)| braid.dataset_id.contains(ds))
            {
                Some((_, exp)) => (found_accession == *exp, exp.to_string()),
                None => (true, String::new()),
            }
        };

        checks.push(BraidCheck {
            braid_id: braid.braid_id.clone(),
            spring: braid.spring.clone(),
            tool,
            substrate,
            accession_ok,
            expected_accession: expected,
            found_accession,
            file: filename.clone(),
        });
    }

    checks
}

/// Display a human-readable summary of braids for CLI output.
#[must_use]
pub fn format_braid_summary(braids: &[(String, FermentBraid)]) -> String {
    if braids.is_empty() {
        return "  No upstream braids found".to_string();
    }

    let mut lines = Vec::new();
    for (filename, braid) in braids {
        let tool_info = if let Some(ref comp) = braid.computation {
            format!("{} ({})", comp.tool, comp.substrate)
        } else if let Some(total) = braid.total_mutations {
            let clones = braid.clones_processed.unwrap_or(0);
            format!("breseq baseline ({total} mutations, {clones} clones)")
        } else {
            "unknown".to_string()
        };

        let detail = if let Some(ref comp) = braid.computation {
            let sv = comp.sovereign_variants.unwrap_or(0);
            let bv = comp.breseq_variants.unwrap_or(0);
            format!("  sovereign={sv} breseq={bv}")
        } else if let Some(ref ref_name) = braid.reference {
            format!("  ref={ref_name}")
        } else {
            String::new()
        };

        lines.push(format!(
            "  {}: {} [{}/{}]{}",
            filename, braid.braid_id, braid.spring, tool_info, detail
        ));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sovereign_braid() {
        let json = r#"{
            "dataset_id": "barrick_2009_sovereign_resequencing",
            "spring": "wetSpring",
            "spring_version": "0.1.0",
            "braid_id": "braid-sovereign-barrick2009",
            "dag_session_id": "dag-wetspring-sovereign-123",
            "computation": {
                "tool": "wetspring-sovereign-pipeline",
                "substrate": "GPU+CPU hybrid",
                "input_accession": "SRP001569",
                "node_count": 7,
                "sovereign_variants": 159,
                "breseq_variants": 569
            }
        }"#;

        let braid: FermentBraid = serde_json::from_str(json).unwrap();
        assert_eq!(braid.braid_id, "braid-sovereign-barrick2009");
        assert_eq!(braid.spring, "wetSpring");

        let comp = braid.computation.as_ref().unwrap();
        assert_eq!(comp.input_accession, "SRP001569");
        assert_eq!(comp.sovereign_variants, Some(159));
        assert_eq!(comp.node_count, Some(7));
    }

    #[test]
    fn parse_breseq_braid() {
        let json = r#"{
            "dataset_id": "barrick_2009",
            "spring": "wetSpring",
            "braid_id": "braid-breseq-barrick2009",
            "clones_processed": 7,
            "total_mutations": 6664,
            "paper": "Barrick et al. Nature 461:1243 (2009)",
            "reference": "CP000819.1",
            "reference_length_bp": 4629812
        }"#;

        let braid: FermentBraid = serde_json::from_str(json).unwrap();
        assert_eq!(braid.total_mutations, Some(6664));
        assert_eq!(braid.clones_processed, Some(7));
        assert!(braid.computation.is_none());
    }

    #[test]
    fn validate_accession_match() {
        let braid = FermentBraid {
            dataset_id: "barrick_2009_sovereign_resequencing".into(),
            spring: "wetSpring".into(),
            spring_version: String::new(),
            braid_id: "braid-sovereign-barrick2009".into(),
            dag_session_id: String::new(),
            dag_merkle_root: String::new(),
            spine_id: String::new(),
            timestamp: String::new(),
            computation: Some(BraidComputation {
                tool: "wetspring-sovereign-pipeline".into(),
                tool_version: String::new(),
                substrate: "GPU+CPU hybrid".into(),
                pipeline: String::new(),
                input_accession: "SRP001569".into(),
                input_blake3: String::new(),
                output_blake3: String::new(),
                wall_time_seconds: None,
                node_count: Some(7),
                sovereign_variants: Some(159),
                breseq_variants: Some(569),
                position_matches: Some(0),
            }),
            clones_processed: None,
            total_mutations: None,
            paper: None,
            reference: None,
            reference_length_bp: None,
            mutation_counts: None,
        };

        let braids = vec![("test.json".into(), braid)];
        let accessions = [("barrick_2009", "SRP001569")];
        let checks = validate_braids(&braids, &accessions);

        assert_eq!(checks.len(), 1);
        assert!(checks[0].accession_ok);
    }

    #[test]
    fn validate_accession_mismatch() {
        let braid = FermentBraid {
            dataset_id: "barrick_2009_sovereign_resequencing".into(),
            spring: "wetSpring".into(),
            spring_version: String::new(),
            braid_id: "braid-sovereign-barrick2009".into(),
            dag_session_id: String::new(),
            dag_merkle_root: String::new(),
            spine_id: String::new(),
            timestamp: String::new(),
            computation: Some(BraidComputation {
                tool: "wetspring-sovereign-pipeline".into(),
                tool_version: String::new(),
                substrate: "GPU+CPU hybrid".into(),
                pipeline: String::new(),
                input_accession: "SRP999999".into(),
                input_blake3: String::new(),
                output_blake3: String::new(),
                wall_time_seconds: None,
                node_count: None,
                sovereign_variants: None,
                breseq_variants: None,
                position_matches: None,
            }),
            clones_processed: None,
            total_mutations: None,
            paper: None,
            reference: None,
            reference_length_bp: None,
            mutation_counts: None,
        };

        let braids = vec![("test.json".into(), braid)];
        let accessions = [("barrick_2009", "SRP001569")];
        let checks = validate_braids(&braids, &accessions);

        assert_eq!(checks.len(), 1);
        assert!(!checks[0].accession_ok);
    }
}
