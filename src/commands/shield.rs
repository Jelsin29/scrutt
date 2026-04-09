use std::path::Path;

use crate::error::ScruttError;
use crate::pkg_json;

pub fn run(path: &Path) -> Result<(), ScruttError> {
    let manifest = pkg_json::load(&path.join("package.json"))?;

    println!(
        "Dependencies: {}, DevDependencies: {}",
        manifest.dependency_count(),
        manifest.dev_dependency_count()
    );

    Ok(())
}
