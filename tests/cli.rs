use std::{fs, process::Command};
use tempfile::TempDir;

fn fixture() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("archive")).unwrap();
    fs::create_dir_all(dir.path().join("target")).unwrap();
    fs::write(
        dir.path().join("app.log"),
        "INFO boot\nERROR Connection timeout\nerror lowercase\n",
    )
    .unwrap();
    fs::write(dir.path().join("auth.txt"), "ERROR Invalid password\n").unwrap();
    fs::write(dir.path().join("archive/old.log"), "ERROR Disk full\n").unwrap();
    fs::write(dir.path().join("target/ignored.log"), "ERROR Hidden\n").unwrap();
    dir
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_my-ripgrep"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn cli_searches_recursively() {
    let dir = fixture();
    let path = dir.path().to_str().unwrap();
    let output = run(&["ERROR", path]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for expected in [
        "ERROR Connection timeout",
        "ERROR Invalid password",
        "ERROR Disk full",
        "ERROR Hidden",
    ] {
        assert!(stdout.contains(expected), "missing {expected}: {stdout}");
    }

    assert!(stdout.contains("Amount files: 4 -- Amount lines: 6 -- Amount matches: 4"));
}

#[test]
fn cli_respects_combined_filters() {
    let dir = fixture();
    let output = run(&[
        "error",
        dir.path().to_str().unwrap(),
        "--ignore-case",
        "--exclude",
        "target",
        "--extension",
        "log",
        "--threads",
        "2",
    ]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("ERROR Connection timeout"));
    assert!(stdout.contains("error lowercase"));
    assert!(stdout.contains("ERROR Disk full"));
    assert!(!stdout.contains("ERROR Hidden"));
    assert!(!stdout.contains("ERROR Invalid password"));
    assert!(stdout.contains("Amount files: 2 -- Amount lines: 4 -- Amount matches: 3"));
}

#[test]
fn cli_searches_one_file() {
    let dir = fixture();
    let path = dir.path().join("app.log");
    let output = run(&["ERROR", path.to_str().unwrap()]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("app.log:2:ERROR Connection timeout"));
    assert!(!stdout.contains("error lowercase"));
}

#[test]
fn cli_fails_when_arguments_are_missing() {
    assert!(!run(&[]).status.success());
    assert!(!run(&["ERROR"]).status.success());
}

#[test]
fn cli_fails_for_nonexistent_path() {
    let dir = fixture();
    let missing = dir.path().join("missing");
    assert!(!run(&["ERROR", missing.to_str().unwrap()]).status.success());
}
