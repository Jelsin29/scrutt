use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum ScruttError {
    #[error("missing package.json: {path}")]
    MissingFile { path: PathBuf },

    #[error("cannot read manifest: {}", .path.as_deref().map_or_else(|| "<unknown>".into(), |path| path.display().to_string()))]
    IoError {
        path: Option<PathBuf>,
        source: io::Error,
    },

    #[error("invalid package.json: {}", .path.as_deref().map_or_else(|| "<unknown>".into(), |path| path.display().to_string()))]
    ParseError {
        path: Option<PathBuf>,
        source: serde_json::Error,
    },
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

#[cfg(test)]
mod tests {
    use super::ScruttError;

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
}
