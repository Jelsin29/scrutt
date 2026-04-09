use std::io;
use std::path::PathBuf;
use std::process::ExitStatus;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NpmrcPatchIssue {
    ConflictingDuplicateKeys,
    NonUtf8Content,
}

#[derive(Debug, thiserror::Error)]
pub enum ScruttError {
    #[error("missing package.json: {path}")]
    MissingFile { path: PathBuf },

    #[error(
        "cannot read manifest {}: {source}",
        .path
            .as_deref()
            .map_or_else(|| "<unknown>".into(), |path| path.display().to_string())
    )]
    IoError {
        path: Option<PathBuf>,
        source: io::Error,
    },

    #[error(
        "invalid package.json {}: {source}",
        .path
            .as_deref()
            .map_or_else(|| "<unknown>".into(), |path| path.display().to_string())
    )]
    ParseError {
        path: Option<PathBuf>,
        source: serde_json::Error,
    },

    #[error("cannot write file {path}: {source}", path = .path.display())]
    WriteError { path: PathBuf, source: io::Error },

    #[error("cannot read text file {path}: {source}", path = .path.display())]
    ReadTextError { path: PathBuf, source: io::Error },

    #[error("npm not found: {source}")]
    MissingBinary {
        program: &'static str,
        source: io::Error,
    },

    #[error(
        "failed to start {program} in {cwd}: {source}",
        cwd = .cwd.display()
    )]
    InstallProcessSpawn {
        program: &'static str,
        cwd: PathBuf,
        source: io::Error,
    },

    #[error(
        "npm installation failed in {cwd}: {status}",
        cwd = .cwd.display(),
        status = render_exit_status(*status)
    )]
    InstallFailed {
        program: &'static str,
        cwd: PathBuf,
        status: ExitStatus,
    },

    #[error(
        "invalid .npmrc state {path}: {reason}",
        path = .path.display(),
        reason = .reason.as_message()
    )]
    InvalidNpmrcState {
        path: PathBuf,
        reason: NpmrcPatchIssue,
    },
}

impl NpmrcPatchIssue {
    pub fn as_message(self) -> &'static str {
        match self {
            Self::ConflictingDuplicateKeys => "conflicting duplicate ignore-scripts entries",
            Self::NonUtf8Content => ".npmrc must be valid UTF-8 text",
        }
    }
}

impl ScruttError {
    pub fn read_failure(path: PathBuf, source: io::Error) -> Self {
        Self::IoError {
            path: Some(path),
            source,
        }
    }

    pub fn invalid_json(path: PathBuf, source: serde_json::Error) -> Self {
        Self::ParseError {
            path: Some(path),
            source,
        }
    }
}

impl From<io::Error> for ScruttError {
    fn from(source: io::Error) -> Self {
        Self::IoError { path: None, source }
    }
}

impl From<serde_json::Error> for ScruttError {
    fn from(source: serde_json::Error) -> Self {
        Self::ParseError { path: None, source }
    }
}

fn render_exit_status(status: ExitStatus) -> String {
    match status.code() {
        Some(code) => format!("exit code {code}"),
        None => "terminated by signal".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::{NpmrcPatchIssue, ScruttError};

    #[test]
    fn converts_io_error_with_generic_context() {
        let error = std::io::Error::other("boom");
        let scrutt_error = ScruttError::from(error);

        match scrutt_error {
            ScruttError::IoError { path, source } => {
                assert!(path.is_none());
                assert_eq!(source.kind(), std::io::ErrorKind::Other);
            }
            other => panic!("expected IoError, got {other:?}"),
        }
    }

    #[test]
    fn converts_json_error_with_generic_context() {
        let error = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let scrutt_error = ScruttError::from(error);

        match scrutt_error {
            ScruttError::ParseError { path, .. } => {
                assert!(path.is_none());
            }
            other => panic!("expected ParseError, got {other:?}"),
        }
    }

    #[test]
    fn renders_invalid_npmrc_state_with_concrete_reason() {
        let error = ScruttError::InvalidNpmrcState {
            path: "tests/fixtures/shield/npmrc_conflicting/.npmrc".into(),
            reason: NpmrcPatchIssue::ConflictingDuplicateKeys,
        };

        assert!(
            error
                .to_string()
                .contains("conflicting duplicate ignore-scripts entries")
        );
    }

    #[test]
    fn renders_install_failed_with_exit_code() {
        let error = ScruttError::InstallFailed {
            program: "npm",
            cwd: "tests/fixtures/valid".into(),
            status: exit_status(23),
        };

        assert!(error.to_string().contains("exit code 23"));
    }

    #[cfg(unix)]
    fn exit_status(code: i32) -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;

        std::process::ExitStatus::from_raw(code << 8)
    }

    #[cfg(windows)]
    fn exit_status(code: u32) -> std::process::ExitStatus {
        use std::os::windows::process::ExitStatusExt;

        std::process::ExitStatus::from_raw(code)
    }
}
