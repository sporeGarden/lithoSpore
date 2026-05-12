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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tolerance_set_get_by_name() {
        let set = ToleranceSet {
            tolerance: vec![
                Tolerance {
                    name: "alpha".into(),
                    value: 0.01,
                    justification: "test".into(),
                },
                Tolerance {
                    name: "beta".into(),
                    value: 0.05,
                    justification: "test".into(),
                },
            ],
        };
        assert_eq!(set.get("alpha").unwrap().value, 0.01);
        assert_eq!(set.get("beta").unwrap().value, 0.05);
        assert!(set.get("gamma").is_none());
    }

    #[test]
    fn tolerance_toml_roundtrip() {
        let toml_str = r#"
[[tolerance]]
name = "test_tol"
value = 0.001
justification = "unit test"
"#;
        let set: ToleranceSet = toml::from_str(toml_str).unwrap();
        assert_eq!(set.tolerance.len(), 1);
        assert_eq!(set.tolerance[0].name, "test_tol");
    }
}
