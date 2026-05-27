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
    pub id: String,
    pub version: String,
    pub spring: String,
    pub tools: Vec<String>,
    pub modules: Vec<ProfileModule>,
    pub figures_enabled: bool,
    pub translation_enabled: bool,
    pub check_commands: Vec<CheckCommand>,
}

/// A module declared in the domain profile.
#[derive(Debug, Clone)]
pub struct ProfileModule {
    pub name: String,
    pub description: String,
    pub check_command: String,
}

/// A domain-specific check command.
#[derive(Debug, Clone)]
pub struct CheckCommand {
    pub name: String,
    pub command: String,
    pub expected_exit: i32,
}

impl DomainProfile {
    /// Load a domain_profile.toml from a file path.
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        Self::parse(&content)
    }

    /// Try to load from a pseudoSpore root directory (looks for domain_profile.toml).
    pub fn from_spore_root(root: &Path) -> Option<Self> {
        let path = root.join("domain_profile.toml");
        if path.exists() {
            Self::load(&path).ok()
        } else {
            None
        }
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
            .unwrap_or("unknown")
            .to_string();

        let tools = profile
            .get("tools")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let figures_enabled = table
            .get("figures")
            .and_then(|v| v.as_table())
            .and_then(|t| t.get("enabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let translation_enabled = table
            .get("translation")
            .and_then(|v| v.as_table())
            .and_then(|t| t.get("enabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

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
                                .and_then(|v| v.as_integer())
                                .unwrap_or(0)
                                as i32,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            id,
            version,
            spring,
            tools,
            modules,
            figures_enabled,
            translation_enabled,
            check_commands,
        })
    }
}
