use std::path::PathBuf;

use assert_cmd::Command;
use predicates::str::contains;

/// A throwaway `HOME` so config/DB-touching commands never read or write the
/// developer's real files (and stay deterministic in CI).
fn isolated_home(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("josephine-it-{}-{tag}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn prints_version() {
    Command::cargo_bin("josephine")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains("josephine"));
}

#[test]
fn help_lists_core_subcommands() {
    Command::cargo_bin("josephine")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("status"))
        .stdout(contains("doctor"))
        .stdout(contains("daemon"));
}

#[test]
fn clean_previews_without_deleting() {
    Command::cargo_bin("josephine")
        .unwrap()
        .env("HOME", isolated_home("clean"))
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .arg("clean")
        .assert()
        .success()
        .stdout(contains("preview"));
}

#[test]
fn help_lists_new_subcommands() {
    Command::cargo_bin("josephine")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("report"))
        .stdout(contains("update"))
        .stdout(contains("notify"));
}

#[test]
fn unknown_command_fails() {
    Command::cargo_bin("josephine")
        .unwrap()
        .arg("definitely-not-a-command")
        .assert()
        .failure();
}

#[test]
fn fix_is_no_longer_a_subcommand() {
    Command::cargo_bin("josephine")
        .unwrap()
        .arg("fix")
        .assert()
        .failure();
}

#[test]
fn help_does_not_offer_fix() {
    // A plain substring check would trip on unrelated future text ("prefix",
    // "fixed", French "correctif"), so look for `fix` as a listed command
    // name specifically — the first word of one of clap's command lines.
    let assert = Command::cargo_bin("josephine")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&assert.get_output().stdout).into_owned();
    let lists_fix = stdout
        .lines()
        .any(|line| line.split_whitespace().next() == Some("fix"));
    assert!(!lists_fix, "help lists a `fix` command:\n{stdout}");
}

#[test]
fn history_runs() {
    // Reads config + SQLite only (no system checks), so it's CI-safe.
    Command::cargo_bin("josephine")
        .unwrap()
        .env("HOME", isolated_home("history"))
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .arg("history")
        .assert()
        .success();
}

#[test]
fn config_validate_accepts_the_default() {
    Command::cargo_bin("josephine")
        .unwrap()
        .env("HOME", isolated_home("config"))
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .args(["config", "validate"])
        .assert()
        .success()
        .stdout(contains("Configuration"));
}

#[test]
fn daemon_help_lists_run() {
    Command::cargo_bin("josephine")
        .unwrap()
        .args(["daemon", "--help"])
        .assert()
        .success()
        .stdout(contains("run"));
}

#[test]
fn status_json_prints_a_json_array_on_stdout() {
    let output = Command::cargo_bin("josephine")
        .unwrap()
        .env("HOME", isolated_home("status-json"))
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .args(["status", "--json"])
        .assert()
        .success()
        .get_output()
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(value.is_array());
}

#[test]
fn help_about_follows_the_configured_language() {
    // English by default (isolated, empty config home → no config file).
    Command::cargo_bin("josephine")
        .unwrap()
        .env("XDG_CONFIG_HOME", isolated_home("help-en"))
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("guardian angel"));

    // French when the config sets `language: fr`.
    let fr = isolated_home("help-fr");
    let cfg = fr.join("josephine");
    std::fs::create_dir_all(&cfg).unwrap();
    std::fs::write(cfg.join("config.yaml"), "language: fr\n").unwrap();
    Command::cargo_bin("josephine")
        .unwrap()
        .env("XDG_CONFIG_HOME", fr)
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("ange gardien"));
}

#[test]
fn explain_lists_checks() {
    Command::cargo_bin("josephine")
        .unwrap()
        .arg("explain")
        .assert()
        .success()
        .stdout(contains("cpu"))
        .stdout(contains("filesystem"));
}

#[test]
fn explain_disk_mentions_disk() {
    Command::cargo_bin("josephine")
        .unwrap()
        .args(["explain", "disk"])
        .assert()
        .success()
        .stdout(contains("disk"));
}

#[test]
fn help_lists_explain_subcommand() {
    Command::cargo_bin("josephine")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("explain"));
}

#[test]
fn completions_generates_a_script() {
    // Completions are generated from the static command tree — no config needed,
    // and must not create any files. Isolated HOME guards against a regression.
    Command::cargo_bin("josephine")
        .unwrap()
        .env("HOME", isolated_home("completions"))
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(contains("josephine"));
}
