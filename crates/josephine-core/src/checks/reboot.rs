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

/// Combine the layered signals. `Some(true)` if any says a reboot is due,
/// `Some(false)` if at least one could answer "no", `None` if nothing could
/// tell (so the check reports "unavailable" instead of guessing).
fn reboot_required() -> Option<bool> {
    let signals = [
        reboot_required_file(),
        needs_restarting(),
        nixos_reboot_pending(),
        kernel_reboot_pending(),
    ];
    if signals.contains(&Some(true)) {
        Some(true)
    } else if signals.iter().any(Option::is_some) {
        Some(false)
    } else {
        None
    }
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
    output.status.code().map(|code| code == 1)
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
