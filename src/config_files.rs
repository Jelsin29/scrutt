use std::fs;
use std::path::Path;

use crate::error::{NpmrcPatchIssue, ScruttError};

const SCRUTT_TOML_BASELINE: &str = "[shield]\nignore_scripts = true\n";
const NPMRC_BASELINE_LINE: &str = "ignore-scripts=true\n";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileChange {
    Created,
    Updated,
    Unchanged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShieldOutcome {
    pub scrutt_toml: FileChange,
    pub npmrc: FileChange,
}

pub fn ensure_scrutt_toml(project_root: &Path) -> Result<FileChange, ScruttError> {
    let path = project_root.join("scrutt.toml");

    if path.exists() {
        return Ok(FileChange::Unchanged);
    }

    fs::write(&path, SCRUTT_TOML_BASELINE).map_err(|source| ScruttError::WriteError {
        path: path.clone(),
        source,
    })?;

    Ok(FileChange::Created)
}

pub fn ensure_npmrc_ignore_scripts(project_root: &Path) -> Result<FileChange, ScruttError> {
    let path = project_root.join(".npmrc");

    if !path.exists() {
        fs::write(&path, NPMRC_BASELINE_LINE).map_err(|source| ScruttError::WriteError {
            path: path.clone(),
            source,
        })?;
        return Ok(FileChange::Created);
    }

    let bytes = fs::read(&path).map_err(|source| ScruttError::ReadTextError {
        path: path.clone(),
        source,
    })?;
    let contents = String::from_utf8(bytes).map_err(|_| ScruttError::InvalidNpmrcState {
        path: path.clone(),
        reason: NpmrcPatchIssue::NonUtf8Content,
    })?;

    let desired = patch_npmrc(&contents, &path)?;

    if desired == contents {
        return Ok(FileChange::Unchanged);
    }

    fs::write(&path, desired).map_err(|source| ScruttError::WriteError {
        path: path.clone(),
        source,
    })?;

    Ok(FileChange::Updated)
}

fn patch_npmrc(contents: &str, path: &Path) -> Result<String, ScruttError> {
    let mut matches = Vec::new();

    for (index, line) in contents.lines().enumerate() {
        if let Some(value) = parse_ignore_scripts_value(line) {
            matches.push((index, value));
        }
    }

    if matches.is_empty() {
        let mut updated = contents.to_owned();

        if !updated.is_empty() && !updated.ends_with('\n') {
            updated.push('\n');
        }

        updated.push_str(NPMRC_BASELINE_LINE);
        return Ok(updated);
    }

    let first_value = matches[0].1;
    let has_conflict = matches
        .iter()
        .skip(1)
        .any(|(_, value)| *value != first_value);
    if has_conflict {
        return Err(ScruttError::InvalidNpmrcState {
            path: path.to_path_buf(),
            reason: NpmrcPatchIssue::ConflictingDuplicateKeys,
        });
    }

    if first_value == "true" {
        return Ok(contents.to_owned());
    }

    let mut lines: Vec<String> = contents.lines().map(ToOwned::to_owned).collect();

    for (index, _) in matches {
        lines[index] = rewrite_ignore_scripts_line(&lines[index]);
    }

    let mut updated = lines.join("\n");
    if contents.ends_with('\n') {
        updated.push('\n');
    }

    Ok(updated)
}

fn parse_ignore_scripts_value(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') || trimmed.starts_with(';') {
        return None;
    }

    let (key, value) = trimmed.split_once('=')?;
    if key.trim() != "ignore-scripts" {
        return None;
    }

    Some(value.trim())
}

fn rewrite_ignore_scripts_line(line: &str) -> String {
    let leading_whitespace_len = line.len() - line.trim_start().len();
    let leading_whitespace = &line[..leading_whitespace_len];
    format!("{leading_whitespace}ignore-scripts=true")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        FileChange, SCRUTT_TOML_BASELINE, ensure_npmrc_ignore_scripts, ensure_scrutt_toml,
    };
    use crate::error::{NpmrcPatchIssue, ScruttError};

    #[test]
    fn creates_scrutt_toml_once() {
        let temp_dir = unique_temp_dir();

        let first = ensure_scrutt_toml(&temp_dir).expect("creates baseline");
        let second = ensure_scrutt_toml(&temp_dir).expect("second run succeeds");

        assert_eq!(first, FileChange::Created);
        assert_eq!(second, FileChange::Unchanged);
        assert_eq!(
            fs::read_to_string(temp_dir.join("scrutt.toml")).expect("reads scrutt.toml"),
            SCRUTT_TOML_BASELINE
        );
    }

    #[test]
    fn updates_existing_false_ignore_scripts() {
        let temp_dir = unique_temp_dir();
        fs::write(
            temp_dir.join(".npmrc"),
            "save-exact=true\nignore-scripts=false\n",
        )
        .expect("writes npmrc");

        let change = ensure_npmrc_ignore_scripts(&temp_dir).expect("updates npmrc");

        assert_eq!(change, FileChange::Updated);
        assert_eq!(
            fs::read_to_string(temp_dir.join(".npmrc")).expect("reads npmrc"),
            "save-exact=true\nignore-scripts=true\n"
        );
    }

    #[test]
    fn rejects_conflicting_duplicate_ignore_scripts_entries() {
        let temp_dir = unique_temp_dir();
        fs::write(
            temp_dir.join(".npmrc"),
            "ignore-scripts=false\nignore-scripts=true\n",
        )
        .expect("writes npmrc");

        let error = ensure_npmrc_ignore_scripts(&temp_dir).unwrap_err();

        match error {
            ScruttError::InvalidNpmrcState { reason, .. } => {
                assert_eq!(reason, NpmrcPatchIssue::ConflictingDuplicateKeys);
            }
            other => panic!("expected InvalidNpmrcState, got {other:?}"),
        }
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time works")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "scrutt-config-files-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).expect("creates temp directory");
        directory
    }
}
