use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn exits_zero_when_node_modules_is_clean() {
    let temp_dir = copy_fixture_dir("tests/fixtures/audit-clean");

    let output = run_audit(&temp_dir);

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
}

#[test]
fn exits_non_zero_and_prints_findings_with_line_numbers() {
    let temp_dir = copy_fixture_dir("tests/fixtures/audit-findings");

    let output = run_audit(&temp_dir);

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "child-process-exec node_modules/shell-pkg/lib/exec.js:2\neval-usage node_modules/sketchy/index.js:3\n"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("audit reported 2 finding(s)"));
}

#[test]
fn exits_non_zero_when_node_modules_is_missing() {
    let output = Command::new(env!("CARGO_BIN_EXE_scrutt"))
        .arg("audit")
        .arg("tests/fixtures/valid")
        .output()
        .expect("binary executes");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("node_modules not found"));
}

#[test]
fn skips_files_larger_than_one_megabyte() {
    let temp_dir = copy_fixture_dir("tests/fixtures/audit-clean");
    let oversized_file = temp_dir.join("node_modules/huge-pkg/index.js");
    let oversized_contents = format!("eval(x)\n{}", "a".repeat(1024 * 1024));

    fs::create_dir_all(oversized_file.parent().expect("parent exists"))
        .expect("creates huge package dir");
    fs::write(&oversized_file, oversized_contents).expect("writes huge file");

    let output = run_audit(&temp_dir);

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).trim().is_empty());
}

#[test]
fn exits_non_zero_when_package_json_is_missing() {
    let output = Command::new(env!("CARGO_BIN_EXE_scrutt"))
        .arg("audit")
        .arg("tests/fixtures/missing")
        .output()
        .expect("binary executes");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing package.json"));
}

#[test]
fn exits_non_zero_when_package_json_is_invalid() {
    let output = Command::new(env!("CARGO_BIN_EXE_scrutt"))
        .arg("audit")
        .arg("tests/fixtures/invalid")
        .output()
        .expect("binary executes");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid package.json"));
}

#[test]
fn detects_findings_in_cjs_and_mjs_files() {
    let temp_dir = copy_fixture_dir("tests/fixtures/audit-clean");
    let cjs_dir = temp_dir.join("node_modules/cjs-pkg");
    let mjs_dir = temp_dir.join("node_modules/mjs-pkg");

    fs::create_dir_all(&cjs_dir).expect("creates cjs package dir");
    fs::create_dir_all(&mjs_dir).expect("creates mjs package dir");
    fs::write(cjs_dir.join("index.cjs"), "eval(data)\n").expect("writes cjs file");
    fs::write(
        mjs_dir.join("index.mjs"),
        "import { exec } from 'node:child_process';\nexec(cmd);\n",
    )
    .expect("writes mjs file");

    let output = run_audit(&temp_dir);

    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("eval-usage node_modules/cjs-pkg/index.cjs:1"));
    assert!(stdout.contains("child-process-exec node_modules/mjs-pkg/index.mjs:2"));
}

fn run_audit(path: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_scrutt"))
        .arg("audit")
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
        std::env::temp_dir().join(format!("scrutt-audit-test-{}-{nanos}", std::process::id()));
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
