// SPDX-License-Identifier: AGPL-3.0-or-later

//! Symlink-based dispatch for lithoSpore.
//!
//! When the `litho` binary is invoked via a symlink (e.g. `./validate`,
//! `./verify`), this module routes directly to the corresponding subcommand
//! without requiring the user to type `litho validate`. This powers the
//! USB artifact's zero-friction entry points.

/// Try to dispatch based on argv\[0\] symlink name.
///
/// Returns `true` if a symlink-based command was dispatched (caller should
/// return immediately). Returns `false` to fall through to full clap parsing.
pub fn try_symlink_dispatch() -> bool {
    let invoked_as = match std::env::args().next().and_then(|a| {
        std::path::Path::new(&a).file_name().map(|f| {
            let name = f.to_string_lossy().to_string();
            name.strip_suffix(".exe").unwrap_or(&name).to_string()
        })
    }) {
        Some(name) => name,
        None => return false,
    };

    let root = ".".to_string();

    match invoked_as.as_str() {
        "validate" => dispatch_validate(&root),
        "verify" => {
            crate::verify::run(&root, false);
            true
        }
        "refresh" => {
            crate::ops::cmd_refresh(&root);
            true
        }
        "spore" | "spore.sh" => {
            dispatch_spore(&root);
            true
        }
        "parity" => {
            let args: Vec<String> = std::env::args().collect();
            let json_out = args.iter().any(|a| a == "--json");
            crate::parity::run(&root, json_out);
            true
        }
        "grow" => dispatch_grow(&root),
        _ => false,
    }
}

fn dispatch_validate(root: &str) -> bool {
    let args: Vec<String> = std::env::args().collect();
    let tier = if args.iter().any(|a| a == "--tier" || a == "--max-tier") {
        args.windows(2)
            .find(|w| w[0] == "--tier" || w[0] == "--max-tier")
            .and_then(|w| w[1].parse::<u8>().ok())
            .unwrap_or(2)
    } else {
        2
    };
    let json_out = args.iter().any(|a| a == "--json");
    crate::validate::run(root, json_out, tier);
    true
}

fn dispatch_spore(root: &str) {
    if std::env::var(litho_core::env_vars::BIOMEOS_ORCHESTRATOR).is_ok() {
        println!("lithoSpore: biomeOS orchestration detected");
        println!("  Spore class: hypogeal-cotyledon");
        println!("  Graph: biomeOS/graphs/lithoSpore_validation.toml");
        return;
    }
    crate::ops::cmd_spore(root);
}

fn dispatch_grow(root: &str) -> bool {
    let args: Vec<String> = std::env::args().collect();
    let container = args.iter().any(|a| a == "--container");
    let target = if let Some(w) = args.windows(2).find(|w| w[0] == "--target") {
        w[1].clone()
    } else if container {
        ".".to_string()
    } else {
        eprintln!("ERROR: --target <DIR> is required for grow");
        eprintln!("Usage: ./grow --target ~/Development/lithoSpore");
        eprintln!("       ./grow --container   (Docker/Podman, any OS)");
        std::process::exit(1);
    };
    let vm = args.iter().any(|a| a == "--vm");
    let ecosystem = args.iter().any(|a| a == "--ecosystem");
    let skip_build = args.iter().any(|a| a == "--skip-build");
    let skip_fetch = args.iter().any(|a| a == "--skip-fetch");
    crate::grow::run(&crate::grow::GrowOptions {
        artifact_root: root,
        target: &target,
        mode: crate::grow::GrowModeFlags {
            vm,
            container,
            ecosystem,
        },
        skip: crate::grow::GrowSkipFlags {
            build: skip_build,
            fetch: skip_fetch,
        },
    });
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unrecognized_binary_name_falls_through() {
        // When invoked as "litho", try_symlink_dispatch should return false
        // (fall through to clap). We can't easily mock argv[0] in a unit test,
        // but we can verify the function doesn't panic on the current binary name.
        let result = try_symlink_dispatch();
        // The test binary name is something like "dispatch-<hash>" — not a known symlink.
        assert!(!result, "unknown binary name should fall through");
    }
}
