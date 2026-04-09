use std::path::Path;

use crate::config_files::{
    FileChange, ShieldOutcome, ensure_npmrc_ignore_scripts, ensure_scrutt_toml,
};
use crate::error::ScruttError;
use crate::pkg_json;

pub fn run(path: &Path) -> Result<(), ScruttError> {
    pkg_json::load(&pkg_json::manifest_path(path))?;

    let outcome = ShieldOutcome {
        scrutt_toml: ensure_scrutt_toml(path)?,
        npmrc: ensure_npmrc_ignore_scripts(path)?,
    };

    println!("{}", render_summary(outcome));

    Ok(())
}

fn render_summary(outcome: ShieldOutcome) -> String {
    let mut parts = Vec::new();

    if let Some(created) = render_group(outcome, FileChange::Created) {
        parts.push(format!("Created: {created}"));
    }

    if let Some(updated) = render_group(outcome, FileChange::Updated) {
        parts.push(format!("Updated: {updated}"));
    }

    if let Some(unchanged) = render_group(outcome, FileChange::Unchanged) {
        parts.push(format!("Unchanged: {unchanged}"));
    }

    parts.join("; ")
}

fn render_group(outcome: ShieldOutcome, target: FileChange) -> Option<String> {
    let mut files = Vec::new();

    if outcome.scrutt_toml == target {
        files.push("scrutt.toml".to_owned());
    }

    if outcome.npmrc == target {
        let npmrc = if target == FileChange::Updated {
            ".npmrc (ignore-scripts=true)"
        } else {
            ".npmrc"
        };
        files.push(npmrc.to_owned());
    }

    if files.is_empty() {
        None
    } else {
        Some(files.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::render_summary;
    use crate::config_files::{FileChange, ShieldOutcome};

    #[test]
    fn renders_summary_in_stable_change_order() {
        let summary = render_summary(ShieldOutcome {
            scrutt_toml: FileChange::Created,
            npmrc: FileChange::Updated,
        });

        assert_eq!(
            summary,
            "Created: scrutt.toml; Updated: .npmrc (ignore-scripts=true)"
        );
    }
}
