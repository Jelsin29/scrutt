use std::process::Command;

#[test]
fn prints_dependency_counts_for_valid_manifest() {
    let output = Command::new(env!("CARGO_BIN_EXE_scrutt"))
        .arg("shield")
        .arg("tests/fixtures/valid")
        .output()
        .expect("binary executes");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "Dependencies: 3, DevDependencies: 2"
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
