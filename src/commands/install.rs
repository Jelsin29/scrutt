use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::ScruttError;
use crate::pkg_json;

const NPM_PROGRAM: &str = "npm";

pub fn run(path: &Path) -> Result<(), ScruttError> {
    pkg_json::load(&pkg_json::manifest_path(path))?;
    run_install(path)
}

fn run_install(path: &Path) -> Result<(), ScruttError> {
    let mut child = spawn_npm(path)?;
    let status = child
        .wait()
        .map_err(|source| ScruttError::InstallProcessSpawn {
            program: NPM_PROGRAM,
            cwd: path.to_path_buf(),
            source,
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(ScruttError::InstallFailed {
            program: NPM_PROGRAM,
            cwd: path.to_path_buf(),
            status,
        })
    }
}

fn spawn_npm(path: &Path) -> Result<std::process::Child, ScruttError> {
    Command::new(NPM_PROGRAM)
        .arg("install")
        .arg("--ignore-scripts")
        .current_dir(path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|source| map_spawn_error(path, source))
}

fn map_spawn_error(path: &Path, source: io::Error) -> ScruttError {
    if source.kind() == io::ErrorKind::NotFound {
        ScruttError::MissingBinary {
            program: NPM_PROGRAM,
            source,
        }
    } else {
        ScruttError::InstallProcessSpawn {
            program: NPM_PROGRAM,
            cwd: path.to_path_buf(),
            source,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::path::Path;

    use super::map_spawn_error;
    use crate::error::ScruttError;

    #[test]
    fn maps_not_found_spawn_errors_to_missing_binary() {
        let error = map_spawn_error(
            Path::new("tests/fixtures/valid"),
            io::Error::from(io::ErrorKind::NotFound),
        );

        match error {
            ScruttError::MissingBinary { program, source } => {
                assert_eq!(program, "npm");
                assert_eq!(source.kind(), io::ErrorKind::NotFound);
            }
            other => panic!("expected MissingBinary, got {other:?}"),
        }
    }

    #[test]
    fn maps_other_spawn_errors_to_install_process_spawn() {
        let error = map_spawn_error(
            Path::new("tests/fixtures/valid"),
            io::Error::from(io::ErrorKind::PermissionDenied),
        );

        match error {
            ScruttError::InstallProcessSpawn {
                program,
                cwd,
                source,
            } => {
                assert_eq!(program, "npm");
                assert_eq!(cwd, Path::new("tests/fixtures/valid"));
                assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
            }
            other => panic!("expected InstallProcessSpawn, got {other:?}"),
        }
    }
}
