//! SMART disk health — asks `smartctl -H` per block device and flags any drive
//! whose self-assessment isn't passing (early warning of an impending failure).
//!
//! `smartctl` usually needs root, so this check is opt-in (see config) and
//! degrades gracefully: missing tool or no read access → an informational
//! "unavailable", never a false alarm.

use std::process::Command;

use anyhow::Result;

use crate::check::{Check, CheckResult, Metric};
use crate::config::SmartCheckConfig;
use crate::i18n::{self, Lang};

pub struct SmartCheck {
    config: SmartCheckConfig,
}

impl SmartCheck {
    pub fn new(config: SmartCheckConfig) -> Self {
        Self { config }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Health {
    Passed,
    Failing,
    Unknown,
}

impl Check for SmartCheck {
    fn name(&self) -> &str {
        "smart"
    }

    fn run(&mut self) -> Result<CheckResult> {
        if !tool_available() {
            return Ok(unavailable(i18n::t(
                "smartmontools not installed (`smartmontools` package)",
                "smartmontools non installé (paquet `smartmontools`)",
            )));
        }

        let devices = block_devices();
        let mut readable = 0;
        let mut failing = 0;
        let mut details = Vec::new();

        for device in &devices {
            match device_health(device) {
                Health::Passed => {
                    readable += 1;
                    details.push(match i18n::lang() {
                        Lang::En => format!("{device}: healthy (SMART OK)"),
                        Lang::Fr => format!("{device} : sain (SMART OK)"),
                    });
                }
                Health::Failing => {
                    readable += 1;
                    failing += 1;
                    details.push(match i18n::lang() {
                        Lang::En => format!("{device}: ⚠ SMART failing — back up!"),
                        Lang::Fr => format!("{device} : ⚠ SMART en échec — sauvegardez !"),
                    });
                }
                Health::Unknown => {}
            }
        }

        if readable == 0 {
            return Ok(unavailable(i18n::t(
                "SMART status unreadable (root required, or disks without SMART)",
                "état SMART illisible (droits root requis, ou disques sans SMART)",
            )));
        }

        // Wear: the worst (highest) "percentage used" across devices that
        // report it. Spinning disks and SMART-less drives simply don't answer.
        let worst_wear = devices
            .iter()
            .filter_map(|device| device_wear(device))
            .fold(None, |acc: Option<f64>, w| {
                Some(acc.map_or(w, |a| a.max(w)))
            });

        if let Some(wear) = worst_wear {
            details.push(match i18n::lang() {
                Lang::En => format!("Most-worn disk: {wear:.0}% of rated writes used."),
                Lang::Fr => format!("Disque le plus usé : {wear:.0}% des écritures prévues."),
            });
        }

        let mut metrics = vec![Metric {
            name: "smart_failing".into(),
            value: failing as f64,
            unit: "disks".into(),
            threshold_warning: Some(1.0),
            threshold_critical: Some(1.0),
        }];
        if let Some(wear) = worst_wear {
            metrics.push(Metric {
                name: "smart_wear_percent".into(),
                value: wear,
                unit: "%".into(),
                threshold_warning: Some(self.config.wear_warning),
                threshold_critical: Some(self.config.wear_critical),
            });
        }

        Ok(CheckResult {
            check_name: "smart".into(),
            metrics,
            details,
            top_processes: vec![],
            status_value: Some(smart_status_value(readable, failing, worst_wear)),
        })
    }
}

/// The compact one-liner for the `status` view: health first, wear appended
/// when a device reports it.
fn smart_status_value(readable: usize, failing: usize, worst_wear: Option<f64>) -> String {
    if failing > 0 {
        return match i18n::lang() {
            Lang::En => format!("⚠ {failing} disk(s) at risk"),
            Lang::Fr => format!("⚠ {failing} disque(s) en alerte"),
        };
    }
    match (i18n::lang(), worst_wear) {
        (Lang::En, Some(wear)) => format!("{readable} healthy, {wear:.0}% worn"),
        (Lang::En, None) => format!("{readable} healthy disk(s)"),
        (Lang::Fr, Some(wear)) => format!("{readable} sain(s), {wear:.0}% d'usure"),
        (Lang::Fr, None) => format!("{readable} disque(s) sain(s)"),
    }
}

/// Read a device's wear as a "percentage used" figure from `smartctl -j -a`.
fn device_wear(device: &str) -> Option<f64> {
    let output = Command::new("smartctl")
        .args(["-j", "-a", device])
        .output()
        .ok()?;
    parse_wear_percent(&String::from_utf8_lossy(&output.stdout))
}

/// Extract wear from `smartctl -j -a` JSON. NVMe exposes `percentage_used`
/// directly; SATA SSDs via a life-left / wearout attribute whose normalized
/// value counts *down* from 100 (new), so wear = 100 - value.
fn parse_wear_percent(json: &str) -> Option<f64> {
    let root: serde_json::Value = serde_json::from_str(json).ok()?;

    if let Some(pct) = root
        .get("nvme_smart_health_information_log")
        .and_then(|log| log.get("percentage_used"))
        .and_then(serde_json::Value::as_f64)
    {
        return Some(pct.clamp(0.0, 100.0));
    }

    let table = root
        .get("ata_smart_attributes")
        .and_then(|a| a.get("table"))
        .and_then(serde_json::Value::as_array)?;
    // Priority: SSD_Life_Left (231), Media_Wearout_Indicator (233),
    // Wear_Leveling_Count (177) — all normalized 100 (new) down to 0.
    for id in [231u64, 233, 177] {
        for attr in table {
            if attr.get("id").and_then(serde_json::Value::as_u64) == Some(id)
                && let Some(normalized) = attr.get("value").and_then(serde_json::Value::as_f64)
            {
                return Some((100.0 - normalized).clamp(0.0, 100.0));
            }
        }
    }
    None
}

fn unavailable(reason: &str) -> CheckResult {
    CheckResult {
        check_name: "smart".into(),
        metrics: vec![],
        details: vec![reason.to_string()],
        top_processes: vec![],
        status_value: Some(i18n::t("Unavailable", "Indisponible").into()),
    }
}

fn tool_available() -> bool {
    Command::new("smartctl")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn block_devices() -> Vec<String> {
    let mut devices = Vec::new();
    let Ok(entries) = std::fs::read_dir("/sys/block") else {
        return devices;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if ["sd", "nvme", "vd", "hd"]
            .iter()
            .any(|p| name.starts_with(p))
        {
            devices.push(format!("/dev/{name}"));
        }
    }
    devices.sort();
    devices
}

fn device_health(device: &str) -> Health {
    match Command::new("smartctl").args(["-H", device]).output() {
        Ok(output) => parse_smart_health(&String::from_utf8_lossy(&output.stdout)),
        Err(_) => Health::Unknown,
    }
}

/// Read the overall-health verdict from `smartctl -H` output (ATA or NVMe).
fn parse_smart_health(stdout: &str) -> Health {
    for line in stdout.lines() {
        let line = line.to_lowercase();
        if line.contains("overall-health") || line.contains("smart health status") {
            if line.contains("passed") || line.contains("ok") {
                return Health::Passed;
            }
            if line.contains("failed") {
                return Health::Failing;
            }
        }
    }
    Health::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ata_passed() {
        let sample = "SMART overall-health self-assessment test result: PASSED\n";
        assert_eq!(parse_smart_health(sample), Health::Passed);
    }

    #[test]
    fn parses_ata_failed() {
        let sample = "SMART overall-health self-assessment test result: FAILED!\n";
        assert_eq!(parse_smart_health(sample), Health::Failing);
    }

    #[test]
    fn parses_nvme_ok() {
        let sample = "SMART Health Status: OK\n";
        assert_eq!(parse_smart_health(sample), Health::Passed);
    }

    #[test]
    fn unknown_when_absent() {
        assert_eq!(parse_smart_health("no verdict here"), Health::Unknown);
    }

    #[test]
    fn parses_nvme_percentage_used() {
        let json = r#"{"nvme_smart_health_information_log": {"percentage_used": 7}}"#;
        assert_eq!(parse_wear_percent(json), Some(7.0));
    }

    #[test]
    fn parses_sata_life_left_as_wear() {
        // SSD_Life_Left normalized value 82 -> 18% worn.
        let json = r#"{"ata_smart_attributes": {"table": [
            {"id": 5, "name": "Reallocated_Sector_Ct", "value": 100},
            {"id": 231, "name": "SSD_Life_Left", "value": 82}
        ]}}"#;
        assert_eq!(parse_wear_percent(json), Some(18.0));
    }

    #[test]
    fn no_wear_attribute_is_none() {
        let json = r#"{"ata_smart_attributes": {"table": [{"id": 5, "value": 100}]}}"#;
        assert_eq!(parse_wear_percent(json), None);
        assert_eq!(parse_wear_percent("not json"), None);
    }

    #[test]
    fn status_value_appends_wear_when_present() {
        use crate::i18n::{self, Lang};
        let _guard = i18n::lock_for_test();
        let prev = i18n::lang();
        i18n::set_lang(Lang::En);
        assert_eq!(smart_status_value(2, 0, None), "2 healthy disk(s)");
        assert_eq!(smart_status_value(2, 0, Some(85.0)), "2 healthy, 85% worn");
        assert_eq!(smart_status_value(2, 1, Some(85.0)), "⚠ 1 disk(s) at risk");
        i18n::set_lang(prev);
    }
}
