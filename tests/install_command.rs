use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn exits_non_zero_when_package_json_is_missing() {
    let output = Command::new(env!("CARGO_BIN_EXE_scrutt"))
        .arg("install")
        .arg("tests/fixtures/missing")
        .output()
        .expect("binary executes");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing package.json"));
}

#[test]
fn runs_npm_install_ignore_scripts_in_target_directory() {
    let temp_dir = copy_fixture_dir("tests/fixtures/valid");
    let bin_dir = unique_temp_dir();
    let args_log = bin_dir.join("args.log");
    let cwd_log = bin_dir.join("cwd.log");
    let npm_path = write_npm_stub(
        &bin_dir,
        r###"#!/bin/sh
printf '%s' "$*" > "$SCRUTT_NPM_ARGS_LOG"
pwd > "$SCRUTT_NPM_CWD_LOG"
printf 'npm stdout ok\n'
printf 'npm stderr ok\n' >&2
exit 0
"###,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_scrutt"))
        .arg("install")
        .arg(&temp_dir)
        .env("PATH", path_with_dir(&bin_dir))
        .env("SCRUTT_NPM_ARGS_LOG", &args_log)
        .env("SCRUTT_NPM_CWD_LOG", &cwd_log)
        .output()
        .expect("binary executes");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(args_log).expect("args log exists"),
        "install --ignore-scripts"
    );
    assert_eq!(
        fs::read_to_string(cwd_log).expect("cwd log exists").trim(),
        temp_dir.to_string_lossy()
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("npm stdout ok"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("npm stderr ok"));

    let _ = npm_path;
}

#[test]
fn exits_non_zero_when_npm_is_missing() {
    let temp_dir = copy_fixture_dir("tests/fixtures/valid");
    let empty_path_dir = unique_temp_dir();

    let output = Command::new(env!("CARGO_BIN_EXE_scrutt"))
        .arg("install")
        .arg(&temp_dir)
        .env("PATH", &empty_path_dir)
        .output()
        .expect("binary executes");

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("npm not found"));
}

#[test]
fn exits_non_zero_when_npm_install_fails() {
    let temp_dir = copy_fixture_dir("tests/fixtures/valid");
    let bin_dir = unique_temp_dir();
    write_npm_stub(
        &bin_dir,
        r###"#!/bin/sh
printf 'npm exploded\n' >&2
exit 23
"###,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_scrutt"))
        .arg("install")
        .arg(&temp_dir)
        .env("PATH", path_with_dir(&bin_dir))
        .output()
        .expect("binary executes");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("npm exploded"));
    assert!(stderr.contains("npm installation failed"));
    assert!(stderr.contains("exit code 23"));
}

fn write_npm_stub(bin_dir: &Path, contents: &str) -> PathBuf {
    fs::create_dir_all(bin_dir).expect("creates bin dir");
    let npm_path = bin_dir.join("npm");
    fs::write(&npm_path, contents).expect("writes npm stub");
    set_executable(&npm_path);
    npm_path
}

fn path_with_dir(dir: &Path) -> std::ffi::OsString {
    match env::var_os("PATH") {
        Some(existing) if !existing.is_empty() => {
            let mut paths = vec![dir.to_path_buf()];
            paths.extend(env::split_paths(&existing));
            env::join_paths(paths).expect("joins PATH")
        }
        _ => dir.as_os_str().to_owned(),
    }
}

#[cfg(unix)]
fn set_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path).expect("reads metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("sets executable permissions");
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) {}

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
    let directory = std::env::temp_dir().join(format!(
        "scrutt-install-test-{}-{nanos}",
        std::process::id()
    ));
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
