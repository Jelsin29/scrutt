use std::fs;
use std::path::{Path, PathBuf};

use crate::error::ScruttError;

const MAX_FILE_SIZE_BYTES: u64 = 1024 * 1024;
const SCANNABLE_EXTENSIONS: &[&str] = &["js", "cjs", "mjs"];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum AuditRule {
    EvalUsage,
    ChildProcessExec,
    ChildProcessSpawn,
}

impl AuditRule {
    fn as_str(self) -> &'static str {
        match self {
            Self::EvalUsage => "eval-usage",
            Self::ChildProcessExec => "child-process-exec",
            Self::ChildProcessSpawn => "child-process-spawn",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditFinding {
    pub rule: AuditRule,
    pub relative_path: PathBuf,
    pub line: usize,
}

impl AuditFinding {
    pub fn render(&self) -> String {
        format!(
            "{} {}:{}",
            self.rule.as_str(),
            self.relative_path.display(),
            self.line
        )
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct AuditReport {
    pub findings: Vec<AuditFinding>,
    pub scanned_files: usize,
}

pub fn scan_project(project_root: &Path) -> Result<AuditReport, ScruttError> {
    let node_modules = project_root.join("node_modules");
    if !node_modules.is_dir() {
        return Err(ScruttError::NodeModulesNotFound { path: node_modules });
    }

    let mut report = AuditReport::default();
    walk_scannable(&node_modules, &node_modules, &mut report)?;
    report.findings.sort_by(|left, right| {
        left.relative_path
            .cmp(&right.relative_path)
            .then(left.line.cmp(&right.line))
            .then(left.rule.cmp(&right.rule))
    });

    Ok(report)
}

fn walk_scannable(
    current_dir: &Path,
    node_modules_root: &Path,
    report: &mut AuditReport,
) -> Result<(), ScruttError> {
    let entries = fs::read_dir(current_dir).map_err(|source| ScruttError::ScanFailed {
        file: current_dir.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| ScruttError::ScanFailed {
            file: current_dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| ScruttError::ScanFailed {
                file: path.clone(),
                source,
            })?;

        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            if is_hidden(&path) {
                continue;
            }

            walk_scannable(&path, node_modules_root, report)?;
            continue;
        }

        if !file_type.is_file() || !is_scannable_extension(&path) {
            continue;
        }

        let metadata = fs::metadata(&path).map_err(|source| ScruttError::ScanFailed {
            file: path.clone(),
            source,
        })?;
        if metadata.len() > MAX_FILE_SIZE_BYTES {
            continue;
        }

        report.scanned_files += 1;
        let contents = fs::read_to_string(&path).map_err(|source| ScruttError::ScanFailed {
            file: path.clone(),
            source,
        })?;
        report
            .findings
            .extend(scan_file(node_modules_root, &path, &contents));
    }

    Ok(())
}

fn scan_file(node_modules_root: &Path, path: &Path, contents: &str) -> Vec<AuditFinding> {
    let relative_path = Path::new("node_modules").join(
        path.strip_prefix(node_modules_root)
            .expect("scanned file stays inside node_modules"),
    );
    let has_child_process_reference =
        contents.contains("child_process") || contents.contains("node:child_process");
    let mut findings = Vec::new();

    for (index, line) in contents.lines().enumerate() {
        let line_number = index + 1;

        if line.contains("eval(") {
            findings.push(AuditFinding {
                rule: AuditRule::EvalUsage,
                relative_path: relative_path.clone(),
                line: line_number,
            });
        }

        if has_child_process_reference && line.contains("exec(") {
            findings.push(AuditFinding {
                rule: AuditRule::ChildProcessExec,
                relative_path: relative_path.clone(),
                line: line_number,
            });
        }

        if has_child_process_reference && line.contains("spawn(") {
            findings.push(AuditFinding {
                rule: AuditRule::ChildProcessSpawn,
                relative_path: relative_path.clone(),
                line: line_number,
            });
        }
    }

    findings
}

pub fn is_scannable_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| SCANNABLE_EXTENSIONS.contains(&extension))
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{is_scannable_extension, scan_file, scan_project, AuditRule};

    #[test]
    fn only_accepts_supported_javascript_extensions() {
        assert!(is_scannable_extension(Path::new("index.js")));
        assert!(is_scannable_extension(Path::new("index.cjs")));
        assert!(is_scannable_extension(Path::new("index.mjs")));
        assert!(!is_scannable_extension(Path::new("index.ts")));
    }

    #[test]
    fn detects_eval_and_child_process_patterns() {
        let findings = scan_file(
            Path::new("node_modules"),
            Path::new("node_modules/pkg/index.js"),
            "import { exec } from \"node:child_process\";\nexec(cmd)\neval(data)\nspawn(x)\n",
        );

        assert_eq!(findings.len(), 3);
        assert_eq!(findings[0].rule, AuditRule::ChildProcessExec);
        assert_eq!(findings[0].line, 2);
        assert_eq!(findings[1].rule, AuditRule::EvalUsage);
        assert_eq!(findings[1].line, 3);
        assert_eq!(findings[2].rule, AuditRule::ChildProcessSpawn);
        assert_eq!(findings[2].line, 4);
    }

    #[test]
    fn skips_hidden_directories() {
        let project_root = unique_temp_dir();
        let hidden_dir = project_root.join("node_modules/.cache");
        fs::create_dir_all(&hidden_dir).expect("creates hidden directory");
        fs::write(project_root.join("package.json"), "{}\n").expect("writes package.json");
        fs::write(hidden_dir.join("hidden.js"), "eval(x)\n").expect("writes hidden file");

        let report = scan_project(&project_root).expect("scan succeeds");

        assert!(report.findings.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn skips_symlinked_entries() {
        use std::os::unix::fs::symlink;

        let project_root = unique_temp_dir();
        let node_modules = project_root.join("node_modules/pkg");
        fs::create_dir_all(&node_modules).expect("creates package directory");
        fs::write(project_root.join("package.json"), "{}\n").expect("writes package.json");

        let source = project_root.join("outside.js");
        fs::write(&source, "eval(x)\n").expect("writes source file");
        symlink(&source, node_modules.join("linked.js")).expect("creates symlink");

        let report = scan_project(&project_root).expect("scan succeeds");

        assert!(report.findings.is_empty());
        assert_eq!(report.scanned_files, 0);
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time works")
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "scrutt-audit-unit-test-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).expect("creates temp directory");
        directory
    }
}
