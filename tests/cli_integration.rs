// tests/cli_integration.rs
//
// Integration tests that invoke the actual compiled binary, the way a real
// user would from a shell. Requires the `assert_cmd`, `predicates`, and
// `tempfile` dev-dependencies (see Cargo.toml snippet in the accompanying
// message).
//
// These tests set HOME / APPDATA to a temp dir via `env` overrides so they
// never touch a real user's blueprint store.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn fiatlux_cmd(config_home: &std::path::Path) -> Command {
    let mut cmd = Command::cargo_bin("fiatlux").expect("fiatlux binary should build");
    // dirs::config_dir() reads %APPDATA% on Windows, $HOME/.config elsewhere.
    cmd.env("APPDATA", config_home);
    cmd.env("HOME", config_home);
    cmd.env("XDG_CONFIG_HOME", config_home);
    cmd
}

#[test]
fn list_reports_empty_store_with_no_blueprints() {
    let home = tempdir().unwrap();

    fiatlux_cmd(home.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("No blueprints imported yet"));
}

#[test]
fn import_then_list_shows_the_blueprint() {
    let home = tempdir().unwrap();
    let source_dir = tempdir().unwrap();
    let source_file = source_dir.path().join("test.fl");
    std::fs::write(
        &source_file,
        "directories { main.js }\nscripts { \"echo hi\" }",
    )
    .unwrap();

    fiatlux_cmd(home.path())
        .args(["import", "mytemplate", source_file.to_str().unwrap()])
        .assert()
        .success();

    fiatlux_cmd(home.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("mytemplate"));
}

#[test]
fn init_fails_clearly_for_unknown_template() {
    let home = tempdir().unwrap();

    fiatlux_cmd(home.path())
        .args(["init", "does-not-exist", "some-project"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no blueprint named"));
}

#[test]
fn init_dry_run_does_not_create_project_dir() {
    let home = tempdir().unwrap();
    let source_dir = tempdir().unwrap();
    let source_file = source_dir.path().join("test.fl");
    std::fs::write(&source_file, "directories { main.js }").unwrap();

    fiatlux_cmd(home.path())
        .args(["import", "mytemplate", source_file.to_str().unwrap()])
        .assert()
        .success();

    let project_dir = source_dir.path().join("my-test-project");

    fiatlux_cmd(home.path())
        .current_dir(source_dir.path())
        .args(["init", "mytemplate", "my-test-project", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[dry-run]"));

    assert!(!project_dir.exists());
}

#[test]
fn init_actually_scaffolds_files() {
    let home = tempdir().unwrap();
    let workdir = tempdir().unwrap();
    let source_file = workdir.path().join("test.fl");
    std::fs::write(
        &source_file,
        "directories { src { main.js } README }\nscripts { }",
    )
    .unwrap();

    fiatlux_cmd(home.path())
        .args(["import", "mytemplate", source_file.to_str().unwrap()])
        .assert()
        .success();

    fiatlux_cmd(home.path())
        .current_dir(workdir.path())
        .args(["init", "mytemplate", "scaffolded-project"])
        .assert()
        .success();

    let project_dir = workdir.path().join("scaffolded-project");
    assert!(project_dir.join("src/main.js").exists());
    assert!(project_dir.join("README.txt").exists());
}
