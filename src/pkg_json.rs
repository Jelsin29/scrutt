use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::Deserialize;

use crate::error::ScruttError;

#[derive(Debug, Deserialize)]
pub struct PackageJson {
    pub name: Option<String>,
    pub dependencies: Option<BTreeMap<String, String>>,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<BTreeMap<String, String>>,
}

impl PackageJson {
    pub fn dependency_count(&self) -> usize {
        self.dependencies.as_ref().map_or(0, BTreeMap::len)
    }

    pub fn dev_dependency_count(&self) -> usize {
        self.dev_dependencies.as_ref().map_or(0, BTreeMap::len)
    }
}

pub fn load(path: &Path) -> Result<PackageJson, ScruttError> {
    let manifest = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Err(ScruttError::MissingFile {
                path: path.to_path_buf(),
            });
        }
        Err(error) => return Err(ScruttError::read_failure(path.to_path_buf(), error)),
    };

    serde_json::from_str(&manifest)
        .map_err(|error| ScruttError::invalid_json(path.to_path_buf(), error))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::load;
    use crate::error::ScruttError;

    #[test]
    fn loads_dependency_counts_from_fixture() {
        let manifest = load(Path::new("tests/fixtures/valid/package.json")).expect("fixture loads");

        assert_eq!(manifest.name.as_deref(), Some("fixture-app"));
        assert_eq!(manifest.dependency_count(), 3);
        assert_eq!(manifest.dev_dependency_count(), 2);
    }

    #[test]
    fn returns_missing_file_error_for_unknown_manifest() {
        let error = load(Path::new("tests/fixtures/missing/package.json")).unwrap_err();

        match error {
            ScruttError::MissingFile { path } => {
                assert!(path.ends_with("tests/fixtures/missing/package.json"));
            }
            other => panic!("expected MissingFile, got {other:?}"),
        }
    }

    #[test]
    fn returns_parse_error_for_invalid_json() {
        let error = load(Path::new("tests/fixtures/invalid/package.json")).unwrap_err();

        match error {
            ScruttError::ParseError {
                path: Some(path), ..
            } => {
                assert!(path.ends_with("tests/fixtures/invalid/package.json"));
            }
            other => panic!("expected ParseError, got {other:?}"),
        }
    }
}
