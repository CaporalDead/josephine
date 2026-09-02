//! Reboot-required check — a kernel or core-library update only takes effect
//! after a restart, security fixes included. This is the quiet gap after
//! `updates` reports "all applied": the packages are installed, but the running
//! system is still on the old code until you reboot.
//!
//! Detection is best-effort and layered, so it works across distros and
//! degrades to an informational "unavailable" rather than a false alarm:
//!   1. Debian/Ubuntu drop a `/run/reboot-required` flag file.
//!   2. Fedora/RHEL ship `needs-restarting -r` (dnf-utils / yum-utils).
//!   3. NixOS: the booted system's kernel/initrd differs from the activated
//!      one (`/run/booted-system` vs `/run/current-system`).
//!   4. Otherwise, compare the running kernel to the newest one installed
//!      under `/lib/modules` (a strictly newer version means a reboot is due).

use std::path::Path;
use std::process::Command;

use anyhow::Result;

use crate::check::{Check, CheckResult, Metric};
use crate::config::RebootCheckConfig;
use crate::i18n::{self, Lang};

pub struct RebootCheck {
    config: RebootCheckConfig,
}

impl RebootCheck {
    pub fn new(config: RebootCheckConfig) -> Self {
        Self { config }
    }
}

impl Check for RebootCheck {
    fn name(&self) -> &str {
        "reboot"
    }

    fn run(&mut self) -> Result<CheckResult> {
        match reboot_required() {
            Some(required) => Ok(build_result(required, &self.config)),
            None => Ok(unavailable()),
        }
    }
}

/// Ask the layered signals in order, stopping at the first "yes".
fn reboot_required() -> Option<bool> {
    let probes: [fn() -> Option<bool>; 4] = [
        reboot_required_file,
        needs_restarting,
        nixos_reboot_pending,
        kernel_reboot_pending,
    ];
    combine(probes.into_iter().map(|probe| probe()))
}

/// Fold the layered signals: the first "yes" wins, and we only answer "no"
/// when something could actually tell — otherwise `None`, and the check
/// reports "unavailable" rather than guessing.
///
/// Takes an iterator so the probes stay **lazy**. `needs-restarting` forks dnf
/// and this runs on every daemon tick, so once `/run/reboot-required` has said
/// yes there is no reason to ask anything else.
fn combine(signals: impl IntoIterator<Item = Option<bool>>) -> Option<bool> {
    let mut answered = false;
    for signal in signals {
        match signal {
            Some(true) => return Some(true),
            Some(false) => answered = true,
            None => {}
        }
    }
    answered.then_some(false)
}

/// Debian/Ubuntu flag file. `Some(true)` when present; `None` when absent
/// (absence alone doesn't prove "no reboot" on a non-Debian system).
fn reboot_required_file() -> Option<bool> {
    let present = Path::new("/run/reboot-required").exists()
        || Path::new("/var/run/reboot-required").exists();
    present.then_some(true)
}

/// Fedora/RHEL `needs-restarting -r`: exit code 1 means a reboot is needed, 0
/// means it isn't. `None` when the tool isn't installed.
fn needs_restarting() -> Option<bool> {
    let output = Command::new("needs-restarting").arg("-r").output().ok()?;
    interpret_needs_restarting(output.status.code())
}

/// Map `needs-restarting -r`'s exit status, and *only* the two codes it
/// documents.
///
/// Anything else — 2 when the RPM database is locked by another dnf, a
/// permission error, 127, a signal — means the tool could not answer. Reading
/// "not 1" as "no reboot needed" turned every one of those into Joséphine
/// calmly reporting "no restart needed": a silent false negative, on the one
/// check whose whole job is to notice a pending security fix.
fn interpret_needs_restarting(code: Option<i32>) -> Option<bool> {
    match code {
        Some(0) => Some(false),
        Some(1) => Some(true),
        _ => None,
    }
}

/// NixOS: a reboot is due when the booted system's kernel or initrd differs
/// from the activated one. Compares the store paths behind `/run/booted-system`
/// and `/run/current-system`; `None` on a non-NixOS system (those paths absent).
fn nixos_reboot_pending() -> Option<bool> {
    let parts = ["kernel", "initrd", "kernel-modules"];
    let read = |base: &str| -> Option<Vec<std::path::PathBuf>> {
        parts
            .iter()
            .map(|p| std::fs::read_link(format!("/run/{base}/{p}")).ok())
            .collect()
    };
    let booted = read("booted-system")?;
    let current = read("current-system")?;
    Some(booted != current)
}

/// Compare the running kernel to the newest installed one. `None` when the
/// kernel release or the module directories can't be read.
fn kernel_reboot_pending() -> Option<bool> {
    let running = std::fs::read_to_string("/proc/sys/kernel/osrelease").ok()?;
    let running = running.trim();
    let mut installed = Vec::new();
    for entry in std::fs::read_dir("/lib/modules").ok()?.flatten() {
        installed.push(entry.file_name().to_string_lossy().into_owned());
    }
    if installed.is_empty() {
        return None;
    }
    Some(newer_kernel_installed(running, &installed))
}

/// Is any installed kernel strictly newer than the running one? Compares the
/// leading numeric version tuple (`6.9.4` in `6.9.4-arch1-1`) only, so a bare
/// packaging-suffix change never raises a false alarm.
fn newer_kernel_installed(running: &str, installed: &[String]) -> bool {
    let running_key = version_key(running);
    installed.iter().any(|k| version_key(k) > running_key)
}

/// The leading dotted-numeric part of a kernel release, as a comparable tuple.
/// Stops at the first segment without a leading digit, so "6.9.4-arch1-1"
/// yields `[6, 9, 4]`.
fn version_key(release: &str) -> Vec<u64> {
    let mut key = Vec::new();
    for part in release.split('.') {
        let digits: String = part.chars().take_while(char::is_ascii_digit).collect();
        match digits.parse::<u64>() {
            Ok(n) => key.push(n),
            Err(_) => break,
        }
    }
    key
}

fn build_result(required: bool, config: &RebootCheckConfig) -> CheckResult {
    let status_value = match (i18n::lang(), required) {
        (Lang::En, true) => "restart needed".to_string(),
        (Lang::En, false) => "no restart needed".to_string(),
        (Lang::Fr, true) => "redémarrage requis".to_string(),
        (Lang::Fr, false) => "aucun redémarrage requis".to_string(),
    };

    let details = vec![match (i18n::lang(), required) {
        (Lang::En, true) => {
            "A restart is needed to finish applying updates (kernel or libraries).".to_string()
        }
        (Lang::En, false) => "No restart is pending.".to_string(),
        (Lang::Fr, true) => {
            "Un redémarrage est nécessaire pour finir d'appliquer des mises à jour (noyau ou bibliothèques).".to_string()
        }
        (Lang::Fr, false) => "Aucun redémarrage en attente.".to_string(),
    }];

    CheckResult {
        check_name: "reboot".into(),
        metrics: vec![Metric {
            name: "reboot_required".into(),
            value: if required { 1.0 } else { 0.0 },
            unit: "flag".into(),
            threshold_warning: Some(config.warning),
            threshold_critical: Some(config.critical),
        }],
        details,
        top_processes: vec![],
        status_value: Some(status_value),
    }
}

fn unavailable() -> CheckResult {
    CheckResult {
        check_name: "reboot".into(),
        metrics: vec![],
        details: vec![
            i18n::t(
                "Reboot status unavailable (no distro flag, `needs-restarting`, or kernel info).",
                "État de redémarrage indisponible (ni indicateur distro, ni `needs-restarting`, ni info noyau).",
            )
            .into(),
        ],
        top_processes: vec![],
        status_value: Some(i18n::t("Unavailable", "Indisponible").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> RebootCheckConfig {
        RebootCheckConfig::default()
    }

    #[test]
    fn version_key_takes_leading_numeric_tuple() {
        assert_eq!(version_key("6.9.4-arch1-1"), vec![6, 9, 4]);
        assert_eq!(version_key("6.10.0-1-generic"), vec![6, 10, 0]);
        assert_eq!(version_key("5.15.0"), vec![5, 15, 0]);
    }

    #[test]
    fn newer_kernel_detected_by_numeric_version() {
        let running = "6.9.4-arch1-1";
        assert!(newer_kernel_installed(
            running,
            &["6.9.4-arch1-1".into(), "6.10.0-arch1-1".into()]
        ));
        // 6.9.10 is newer than 6.9.4 numerically (not lexically).
        assert!(newer_kernel_installed(running, &["6.9.10-arch1-1".into()]));
    }

    #[test]
    fn same_kernel_with_suffix_change_is_not_flagged() {
        // Only the numeric tuple counts, so a packaging-suffix bump is quiet.
        assert!(!newer_kernel_installed(
            "6.9.4-arch1-1",
            &["6.9.4-arch1-1".into(), "6.9.4-arch1-2".into()]
        ));
    }

    #[test]
    fn only_the_running_kernel_installed_is_not_flagged() {
        assert!(!newer_kernel_installed(
            "6.9.4-arch1-1",
            &["6.9.4-arch1-1".into()]
        ));
    }

    /// The regression this guards: any exit code other than 0 or 1 means the
    /// tool failed, not that the machine is fine.
    #[test]
    fn needs_restarting_only_trusts_its_two_documented_codes() {
        assert_eq!(interpret_needs_restarting(Some(0)), Some(false));
        assert_eq!(interpret_needs_restarting(Some(1)), Some(true));
        // 2 = another dnf holds the lock; 13 = permission denied; 127 = no
        // such command; None = killed by a signal. None of these is an answer.
        for code in [2, 13, 100, 127, 255] {
            assert_eq!(
                interpret_needs_restarting(Some(code)),
                None,
                "exit code {code} must not be read as an answer"
            );
        }
        assert_eq!(interpret_needs_restarting(None), None);
    }

    #[test]
    fn combine_prefers_yes_then_no_then_unknown() {
        assert_eq!(combine([None, Some(false), Some(true)]), Some(true));
        assert_eq!(combine([None, Some(false)]), Some(false));
        assert_eq!(combine([None, None]), None);
        assert_eq!(combine([]), None);
    }

    /// A "yes" must stop the walk: the probes after it fork dnf, and this runs
    /// on every daemon tick.
    #[test]
    fn a_yes_stops_the_remaining_probes() {
        let asked = std::cell::Cell::new(0);
        let signals = [Some(true), Some(false), None];
        let verdict = combine(signals.into_iter().inspect(|_| asked.set(asked.get() + 1)));
        assert_eq!(verdict, Some(true));
        assert_eq!(asked.get(), 1, "probes after the first yes must not run");
    }

    #[test]
    fn required_is_attention_not_critical() {
        let result = build_result(true, &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Attention);
        assert_eq!(result.status_value.as_deref(), Some("restart needed"));
    }

    #[test]
    fn not_required_is_info() {
        let result = build_result(false, &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Info);
        assert_eq!(result.status_value.as_deref(), Some("no restart needed"));
    }
}
