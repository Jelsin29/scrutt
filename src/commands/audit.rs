use std::path::Path;

use crate::audit;
use crate::error::ScruttError;
use crate::pkg_json;

pub fn run(path: &Path) -> Result<(), ScruttError> {
    pkg_json::load(&pkg_json::manifest_path(path))?;

    let report = audit::scan_project(path)?;

    for finding in &report.findings {
        println!("{}", finding.render());
    }

    if report.findings.is_empty() {
        Ok(())
    } else {
        Err(ScruttError::AuditFindings {
            count: report.findings.len(),
        })
    }
}
