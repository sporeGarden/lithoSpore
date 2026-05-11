// SPDX-License-Identifier: AGPL-3.0-or-later

//! Named tolerances with scientific justification.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tolerance {
    pub name: String,
    pub value: f64,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToleranceSet {
    pub tolerance: Vec<Tolerance>,
}

impl ToleranceSet {
    /// Load from a TOML file path.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    pub fn load(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        let set: Self = toml::from_str(&contents)?;
        Ok(set)
    }

    /// Look up a tolerance by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Tolerance> {
        self.tolerance.iter().find(|t| t.name == name)
    }
}
