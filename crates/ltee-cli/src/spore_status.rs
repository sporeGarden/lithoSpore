// SPDX-License-Identifier: AGPL-3.0-or-later

//! `litho spore-status` — registry dashboard for pseudoSpore emission pipeline.
//!
//! Shows the status of all registered pseudoSpores with what each spring team
//! needs to do to move from PENDING to COMPLETE.

use std::path::Path;

pub fn run(artifact_root: &str, json: bool) {
    let registry_path = Path::new(artifact_root).join("pseudospores/registry.toml");
    let content = match std::fs::read_to_string(&registry_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Cannot read registry: {e}");
            eprintln!("  Expected: {}", registry_path.display());
            std::process::exit(1);
        }
    };

    let registry: toml::Value = match toml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Cannot parse registry: {e}");
            std::process::exit(1);
        }
    };

    let entries = registry
        .get("pseudospore")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let total = entries.len();
    let complete = entries
        .iter()
        .filter(|e| e.get("status").and_then(toml::Value::as_str) == Some("COMPLETE"))
        .count();
    let pending = total - complete;

    if json {
        print_json(&entries, total, complete, pending);
    } else {
        print_table(&entries, total, complete, pending);
    }
}

fn str_field<'a>(entry: &'a toml::Value, key: &str) -> &'a str {
    entry.get(key).and_then(toml::Value::as_str).unwrap_or("?")
}

fn int_field(entry: &toml::Value, key: &str) -> i64 {
    entry
        .get(key)
        .and_then(toml::Value::as_integer)
        .unwrap_or(0)
}

fn print_json(entries: &[toml::Value], total: usize, complete: usize, pending: usize) {
    let spores: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "name": str_field(e, "name"),
                "version": str_field(e, "version"),
                "spring": str_field(e, "spring"),
                "status": str_field(e, "status"),
                "modules_pass": int_field(e, "modules_pass"),
                "modules_total": int_field(e, "modules_total"),
                "date": str_field(e, "date"),
            })
        })
        .collect();

    let report = serde_json::json!({
        "total": total,
        "complete": complete,
        "pending": pending,
        "spores": spores,
    });
    match serde_json::to_string_pretty(&report) {
        Ok(s) => println!("{s}"),
        Err(e) => eprintln!("JSON serialization error: {e}"),
    }
}

fn print_table(entries: &[toml::Value], total: usize, complete: usize, pending: usize) {
    println!("=== pseudoSpore Registry Status ===\n");
    println!("  Total: {total}  |  COMPLETE: {complete}  |  PENDING: {pending}\n");
    println!(
        "  {:<40} {:<10} {:<10} {:<12} {:<10}",
        "Name", "Version", "Spring", "Status", "Modules"
    );
    println!("  {}", "-".repeat(90));

    for entry in entries {
        let name = str_field(entry, "name");
        let version = str_field(entry, "version");
        let spring = str_field(entry, "spring");
        let status = str_field(entry, "status");
        let pass = int_field(entry, "modules_pass");
        let total_mods = int_field(entry, "modules_total");

        let modules_str = if total_mods == 0 && pass == 0 {
            "awaiting validation".to_string()
        } else {
            format!("{pass}/{total_mods}")
        };

        println!("  {name:<40} {version:<10} {spring:<10} {status:<12} {modules_str}");
    }

    if pending > 0 {
        println!("\n  Next steps for PENDING spores:");
        println!("    1. litho init-validation <spore> → generate validation.json from scope.toml");
        println!("    2. Spring team runs validators → produces module results JSON");
        println!("    3. litho populate-validation <spore> --results <results.json>");
        println!("    4. litho audit --path <spore> → verify all modules PASS");
        println!("    5. litho promote-spore <spore> → PENDING → COMPLETE");
        println!("    6. litho pack-pseudospore <spore> → redistribute");
    }

    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_table_handles_empty_registry() {
        print_table(&[], 0, 0, 0);
    }

    #[test]
    fn print_json_handles_empty_registry() {
        print_json(&[], 0, 0, 0);
    }
}
