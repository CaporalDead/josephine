//! SMART disk health — asks `smartctl -j -a` once per block device, flags any
//! drive whose self-assessment isn't passing (early warning of an impending
//! failure), and reports how worn the most-used SSD is.
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
        // Wear: the worst (highest) "percentage used" across devices that
        // report it. Spinning disks and SMART-less drives simply don't answer.
        let mut worst_wear: Option<f64> = None;

        for device in &devices {
            let (health, wear) = device_report(device);
            if let Some(wear) = wear {
                worst_wear = Some(worst_wear.map_or(wear, |worst: f64| worst.max(wear)));
            }
            match health {
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

/// Normalized SMART attributes that count *down* from 100 (new) to 0
/// (exhausted), most specific first.
///
/// Matched by **name**, never by id alone: vendors reuse the same id for
/// unrelated things — 231 is `SSD_Life_Left` on some drives and
/// `Temperature_Celsius` on Crucial/Micron, and smartmontools is what
/// disambiguates them per model. Reading id 231 blindly turns a disk running
/// at 70 °C into "30% worn", which is exactly the kind of invented alarm this
/// project promises not to raise.
const LIFE_LEFT_ATTRIBUTES: &[&str] = &[
    "ssd_life_left",
    "media_wearout_indicator",
    "wear_leveling_count",
    "percent_lifetime_remain",
    "remaining_lifetime_perc",
];

/// One `smartctl -j -a` per device, read for both the health verdict and the
/// wear figure.
///
/// `-a` is a superset of `-H` and its JSON already carries `smart_status`, so
/// asking twice only doubled the forks — and this check runs on every daemon
/// tick, against drives that may be spun down.
fn device_report(device: &str) -> (Health, Option<f64>) {
    match Command::new("smartctl").args(["-j", "-a", device]).output() {
        Ok(output) => parse_report(&String::from_utf8_lossy(&output.stdout)),
        Err(_) => (Health::Unknown, None),
    }
}

/// Parse one `smartctl -j -a` payload into (health, wear). Anything we can't
/// read stays `Unknown`/`None` — never a verdict we didn't actually get.
fn parse_report(stdout: &str) -> (Health, Option<f64>) {
    let Ok(root) = serde_json::from_str::<serde_json::Value>(stdout) else {
        return (Health::Unknown, None);
    };
    (parse_smart_health(&root), parse_wear_percent(&root))
}

/// The overall-health verdict. `smart_status.passed` covers both ATA and NVMe,
/// which the text output spelled two different ways.
fn parse_smart_health(root: &serde_json::Value) -> Health {
    match root
        .get("smart_status")
        .and_then(|status| status.get("passed"))
        .and_then(serde_json::Value::as_bool)
    {
        Some(true) => Health::Passed,
        Some(false) => Health::Failing,
        None => Health::Unknown,
    }
}

/// Extract wear from `smartctl -j -a` JSON. NVMe exposes `percentage_used`
/// directly; SATA SSDs via a life-left attribute whose normalized value counts
/// *down* from 100 (new), so wear = 100 - value.
fn parse_wear_percent(root: &serde_json::Value) -> Option<f64> {
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
    for wanted in LIFE_LEFT_ATTRIBUTES {
        for attr in table {
            let Some(name) = attr.get("name").and_then(serde_json::Value::as_str) else {
                continue;
            };
            if name.eq_ignore_ascii_case(wanted)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Health and wear both come out of one payload now.
    fn wear(json: &str) -> Option<f64> {
        parse_report(json).1
    }

    #[test]
    fn parses_passed() {
        let json = r#"{"smart_status": {"passed": true}}"#;
        assert_eq!(parse_report(json).0, Health::Passed);
    }

    #[test]
    fn parses_failed() {
        let json = r#"{"smart_status": {"passed": false}}"#;
        assert_eq!(parse_report(json).0, Health::Failing);
    }

    #[test]
    fn unknown_when_absent_or_unreadable() {
        assert_eq!(parse_report(r#"{"device": {}}"#).0, Health::Unknown);
        assert_eq!(parse_report("no verdict here"), (Health::Unknown, None));
        assert_eq!(parse_report(""), (Health::Unknown, None));
    }

    #[test]
    fn parses_nvme_percentage_used() {
        let json = r#"{"nvme_smart_health_information_log": {"percentage_used": 7}}"#;
        assert_eq!(wear(json), Some(7.0));
    }

    #[test]
    fn parses_sata_life_left_as_wear() {
        // SSD_Life_Left normalized value 82 -> 18% worn.
        let json = r#"{"ata_smart_attributes": {"table": [
            {"id": 5, "name": "Reallocated_Sector_Ct", "value": 100},
            {"id": 231, "name": "SSD_Life_Left", "value": 82}
        ]}}"#;
        assert_eq!(wear(json), Some(18.0));
    }

    /// The bug this guards: on Crucial/Micron drives id 231 is the temperature,
    /// so matching by id turned a 70 °C disk into "30% worn".
    #[test]
    fn id_231_as_temperature_is_not_read_as_wear() {
        let json = r#"{"ata_smart_attributes": {"table": [
            {"id": 231, "name": "Temperature_Celsius", "value": 70}
        ]}}"#;
        assert_eq!(wear(json), None);
    }

    /// An attribute with no resolved name is ambiguous, so it is left alone.
    #[test]
    fn an_unnamed_attribute_is_never_guessed_at() {
        let json = r#"{"ata_smart_attributes": {"table": [{"id": 231, "value": 70}]}}"#;
        assert_eq!(wear(json), None);
    }

    #[test]
    fn recognises_the_other_life_left_attributes() {
        for name in ["Media_Wearout_Indicator", "Wear_Leveling_Count"] {
            let json = format!(
                r#"{{"ata_smart_attributes": {{"table": [
                    {{"id": 1, "name": "{name}", "value": 95}}
                ]}}}}"#
            );
            assert_eq!(wear(&json), Some(5.0), "{name}");
        }
    }

    #[test]
    fn no_wear_attribute_is_none() {
        let json = r#"{"ata_smart_attributes": {"table": [
            {"id": 5, "name": "Reallocated_Sector_Ct", "value": 100}
        ]}}"#;
        assert_eq!(wear(json), None);
        assert_eq!(wear("not json"), None);
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
