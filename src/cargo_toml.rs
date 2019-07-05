use failure::{Error, ResultExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// The `[lib]` section of a Cargo.toml
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CargoTomlLib {
    pub(crate) crate_type: Option<Vec<String>>,
    pub(crate) name: Option<String>,
}

/// The `[package]` section of a Cargo.toml
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CargoTomlPackage {
    // Those three fields are mandatory
    // https://doc.rust-lang.org/cargo/reference/manifest.html#the-package-section
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) authors: Vec<String>,
    // All other fields are optional
    pub(crate) description: Option<String>,
    pub(crate) documentation: Option<String>,
    pub(crate) homepage: Option<String>,
    pub(crate) repository: Option<String>,
    pub(crate) readme: Option<String>,
    pub(crate) keywords: Option<Vec<String>>,
    pub(crate) categories: Option<Vec<String>>,
    pub(crate) license: Option<String>,
    metadata: Option<CargoTomlMetadata>,
}

/// Extract of the Cargo.toml that can be reused for the python metadata
///
/// See https://doc.rust-lang.org/cargo/reference/manifest.html for a specification
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct CargoToml {
    pub(crate) lib: Option<CargoTomlLib>,
    pub(crate) package: CargoTomlPackage,
}

impl CargoToml {
    /// Reads and parses the Cargo.toml at the given location
    pub fn from_path(manifest_file: impl AsRef<Path>) -> Result<Self, Error> {
        let contents = fs::read_to_string(&manifest_file).context(format!(
            "Can't read Cargo.toml at {}",
            manifest_file.as_ref().display(),
        ))?;
        let cargo_toml = toml::from_str(&contents).context(format!(
            "Failed to parse Cargo.toml at {}",
            manifest_file.as_ref().display()
        ))?;
        Ok(cargo_toml)
    }

    /// Returns the python entrypoints
    pub fn scripts(&self) -> HashMap<String, String> {
        match self.package.metadata {
            Some(CargoTomlMetadata {
                pyo3_pack:
                    Some(RemainingCoreMetadata {
                        scripts: Some(ref scripts),
                        ..
                    }),
            }) => scripts.clone(),
            _ => HashMap::new(),
        }
    }

    /// Returns the trove classifier
    pub fn classifier(&self) -> Vec<String> {
        match self.package.metadata {
            Some(CargoTomlMetadata {
                pyo3_pack:
                    Some(RemainingCoreMetadata {
                        classifier: Some(ref classifier),
                        ..
                    }),
            }) => classifier.clone(),
            _ => Vec::new(),
        }
    }

    /// Returns the value of `[project.metadata.pyo3-pack]` or an empty stub
    pub fn remaining_core_metadata(&self) -> RemainingCoreMetadata {
        match &self.package.metadata {
            Some(CargoTomlMetadata {
                pyo3_pack: Some(extra_metadata),
            }) => extra_metadata.clone(),
            _ => Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
struct CargoTomlMetadata {
    pyo3_pack: Option<RemainingCoreMetadata>,
}

/// The `[project.metadata.pyo3-pack]` with the python specific metadata
///
/// Those fields are the part of the
/// [python core metadata](https://packaging.python.org/specifications/core-metadata/)
/// that doesn't have an equivalent in cargo's `[package]` table
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct RemainingCoreMetadata {
    pub scripts: Option<HashMap<String, String>>,
    pub classifier: Option<Vec<String>>,
    pub maintainer: Option<String>,
    pub maintainer_email: Option<String>,
    pub requires_dist: Option<Vec<String>>,
    pub requires_python: Option<String>,
    pub requires_external: Option<Vec<String>>,
    pub project_url: Option<Vec<String>>,
    pub provides_extra: Option<Vec<String>>,
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;
    use toml;

    #[test]
    fn test_metadata_from_cargo_toml() {
        let cargo_toml = indoc!(
            r#"
            [package]
            authors = ["konstin <konstin@mailbox.org>"]
            name = "info-project"
            version = "0.1.0"
            description = "A test project"
            homepage = "https://example.org"
            keywords = ["ffi", "test"]

            [lib]
            crate-type = ["cdylib"]
            name = "pyo3_pure"

            [package.metadata.pyo3-pack.scripts]
            ph = "pyo3_pack:print_hello"

            [package.metadata.pyo3-pack]
            classifier = ["Programming Language :: Python"]
            requires-dist = ["flask~=1.1.0", "toml==0.10.0"]
        "#
        );

        let cargo_toml: CargoToml = toml::from_str(&cargo_toml).unwrap();

        let mut scripts = HashMap::new();
        scripts.insert("ph".to_string(), "pyo3_pack:print_hello".to_string());

        let classifier = vec!["Programming Language :: Python".to_string()];

        let requires_dist = Some(vec!["flask~=1.1.0".to_string(), "toml==0.10.0".to_string()]);

        assert_eq!(cargo_toml.scripts(), scripts);
        assert_eq!(cargo_toml.classifier(), classifier);
        assert_eq!(
            cargo_toml.remaining_core_metadata().requires_dist,
            requires_dist
        );
    }
}
