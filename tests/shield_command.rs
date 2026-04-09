use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const SCRUTT_TOML_BASELINE: &str = "[shield]\nignore_scripts = true\n";

#[test]
fn creates_baseline_files_when_missing() {
    let temp_dir = copy_fixture_dir("tests/fixtures/shield/baseline");

    let output = run_shield(&temp_dir);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Created: scrutt.toml, .npmrc"
    );
    assert_eq!(
        fs::read_to_string(temp_dir.join("scrutt.toml")).expect("scrutt.toml exists"),
        SCRUTT_TOML_BASELINE
    );
    assert_eq!(
        fs::read_to_string(temp_dir.join(".npmrc")).expect(".npmrc exists"),
        "ignore-scripts=true\n"
    );
}

#[test]
fn updates_npmrc_and_preserves_existing_scrutt_toml() {
    let temp_dir = copy_fixture_dir("tests/fixtures/shield/existing_scrutt");
    fs::write(
        temp_dir.join(".npmrc"),
        "save-exact=true\nignore-scripts=false\n",
    )
    .expect("writes .npmrc fixture");

    let output = run_shield(&temp_dir);

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Updated: .npmrc (ignore-scripts=true); Unchanged: scrutt.toml"
    );
    assert_eq!(
        fs::read_to_string(temp_dir.join("scrutt.toml")).expect("scrutt.toml exists"),
        "mode = \"existing\"\n"
    );
    assert_eq!(
        fs::read_to_string(temp_dir.join(".npmrc")).expect(".npmrc exists"),
        "save-exact=true\nignore-scripts=true\n"
    );
}

#[test]
fn reports_unchanged_when_baseline_is_already_present() {
    let temp_dir = copy_fixture_dir("tests/fixtures/shield/npmrc_present");
    fs::write(temp_dir.join("scrutt.toml"), SCRUTT_TOML_BASELINE).expect("writes scrutt.toml");

    let first_output = run_shield(&temp_dir);
    assert!(first_output.status.success());

    let second_output = run_shield(&temp_dir);

    assert!(second_output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&second_output.stdout).trim(),
        "Unchanged: scrutt.toml, .npmrc"
    );
}

#[test]
fn exits_non_zero_for_conflicting_npmrc_entries() {
    let temp_dir = copy_fixture_dir("tests/fixtures/shield/npmrc_conflicting");

    let output = run_shield(&temp_dir);

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("conflicting duplicate ignore-scripts entries")
    );
}

#[test]
fn exits_non_zero_when_package_json_is_missing() {
    let output = Command::new(env!("CARGO_BIN_EXE_scrutt"))
        .arg("shield")
        .arg("tests/fixtures/missing")
        .output()
        .expect("binary executes");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing package.json"));
}

#[test]
fn exits_non_zero_when_package_json_is_invalid() {
    let output = Command::new(env!("CARGO_BIN_EXE_scrutt"))
        .arg("shield")
        .arg("tests/fixtures/invalid")
        .output()
        .expect("binary executes");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid package.json"));
}

fn run_shield(path: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_scrutt"))
        .arg("shield")
        .arg(path)
        .output()
        .expect("binary executes")
}

fn copy_fixture_dir(source: &str) -> PathBuf {
    let source = Path::new(source);
    let destination = unique_temp_dir();
    copy_dir_all(source, &destination);
    destination
}

fn unique_temp_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time works")
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("scrutt-shield-test-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&directory).expect("creates temp directory");
    directory
}

fn copy_dir_all(source: &Path, destination: &Path) {
    for entry in fs::read_dir(source).expect("reads fixture directory") {
        let entry = entry.expect("reads fixture entry");
        let entry_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if entry.file_type().expect("reads fixture file type").is_dir() {
            fs::create_dir_all(&destination_path).expect("creates nested fixture directory");
            copy_dir_all(&entry_path, &destination_path);
        } else {
            fs::copy(&entry_path, &destination_path).expect("copies fixture file");
        }
    }
}
