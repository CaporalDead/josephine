//! Pressure Stall Information (PSI) — the modern, honest signal that the
//! machine is *struggling*, well before a raw usage percentage looks alarming.
//! Reads `/proc/pressure/{memory,cpu,io}` and takes the `some avg60` figure:
//! the share of the last 60 seconds during which at least one task was stalled
//! waiting for that resource.
//!
//! Memory pressure is the one that carries a threshold — a rising memory PSI is
//! the earliest warning of a swap death-spiral, ahead of the OOM killer. CPU
//! and IO pressure are recorded too (for history and `doctor`), but don't
//! raise alerts on their own. Degrades to "unavailable" where PSI isn't exposed
//! (older kernels, or `CONFIG_PSI` disabled).

use anyhow::Result;

use crate::check::{Check, CheckResult, Metric};
use crate::config::PressureCheckConfig;
use crate::i18n::{self, Lang};

pub struct PressureCheck {
    config: PressureCheckConfig,
}

impl PressureCheck {
    pub fn new(config: PressureCheckConfig) -> Self {
        Self { config }
    }
}

impl Check for PressureCheck {
    fn name(&self) -> &str {
        "pressure"
    }

    fn run(&mut self) -> Result<CheckResult> {
        let Some(memory) = read_some_avg60("memory") else {
            return Ok(unavailable());
        };
        let cpu = read_some_avg60("cpu");
        let io = read_some_avg60("io");
        Ok(build_result(memory, cpu, io, &self.config))
    }
}

fn build_result(
    memory: f64,
    cpu: Option<f64>,
    io: Option<f64>,
    config: &PressureCheckConfig,
) -> CheckResult {
    let status_value = match i18n::lang() {
        Lang::En => format!("{memory:.0}% memory (60s)"),
        Lang::Fr => format!("{memory:.0}% mémoire (60s)"),
    };

    let mut details = vec![match i18n::lang() {
        Lang::En => format!("Memory stall: {memory:.1}% of the last minute."),
        Lang::Fr => format!("Blocage mémoire : {memory:.1}% de la dernière minute."),
    }];
    if let Some(cpu) = cpu {
        details.push(match i18n::lang() {
            Lang::En => format!("CPU stall: {cpu:.1}%."),
            Lang::Fr => format!("Blocage CPU : {cpu:.1}%."),
        });
    }
    if let Some(io) = io {
        details.push(match i18n::lang() {
            Lang::En => format!("IO stall: {io:.1}%."),
            Lang::Fr => format!("Blocage E/S : {io:.1}%."),
        });
    }

    let mut metrics = vec![Metric {
        name: "memory_pressure_avg60".into(),
        value: memory,
        unit: "%".into(),
        threshold_warning: Some(config.warning),
        threshold_critical: Some(config.critical),
    }];
    // CPU and IO are informational: recorded for history, no thresholds so the
    // rules engine never alerts on them.
    if let Some(cpu) = cpu {
        metrics.push(Metric {
            name: "cpu_pressure_avg60".into(),
            value: cpu,
            unit: "%".into(),
            threshold_warning: None,
            threshold_critical: None,
        });
    }
    if let Some(io) = io {
        metrics.push(Metric {
            name: "io_pressure_avg60".into(),
            value: io,
            unit: "%".into(),
            threshold_warning: None,
            threshold_critical: None,
        });
    }

    CheckResult {
        check_name: "pressure".into(),
        metrics,
        details,
        top_processes: vec![],
        status_value: Some(status_value),
    }
}

fn unavailable() -> CheckResult {
    CheckResult {
        check_name: "pressure".into(),
        metrics: vec![],
        details: vec![
            i18n::t(
                "Pressure stall info unavailable (`/proc/pressure` — kernel without PSI).",
                "Info de pression indisponible (`/proc/pressure` — noyau sans PSI).",
            )
            .into(),
        ],
        top_processes: vec![],
        status_value: Some(i18n::t("Unavailable", "Indisponible").into()),
    }
}

/// Read the `some avg60` percentage for a PSI resource (`memory`, `cpu`, `io`).
fn read_some_avg60(resource: &str) -> Option<f64> {
    let content = std::fs::read_to_string(format!("/proc/pressure/{resource}")).ok()?;
    parse_some_avg60(&content)
}

/// Parse the `some` line of a PSI file and return its `avg60` value.
fn parse_some_avg60(content: &str) -> Option<f64> {
    for line in content.lines() {
        if let Some(rest) = line.trim().strip_prefix("some ") {
            for field in rest.split_whitespace() {
                if let Some(value) = field.strip_prefix("avg60=") {
                    return value.parse::<f64>().ok();
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
some avg10=0.12 avg60=3.40 avg300=0.27 total=5866198773
full avg10=0.00 avg60=0.00 avg300=0.00 total=0
";

    fn config() -> PressureCheckConfig {
        PressureCheckConfig::default()
    }

    #[test]
    fn parses_some_avg60() {
        assert_eq!(parse_some_avg60(SAMPLE), Some(3.40));
    }

    #[test]
    fn missing_some_line_is_none() {
        assert_eq!(parse_some_avg60("full avg60=1.0 total=0\n"), None);
    }

    #[test]
    fn calm_memory_is_info() {
        let result = build_result(0.5, Some(0.1), Some(0.0), &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Info);
    }

    #[test]
    fn high_memory_pressure_is_critical() {
        let result = build_result(55.0, Some(1.0), None, &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Critique);
    }

    #[test]
    fn cpu_and_io_metrics_carry_no_thresholds() {
        let result = build_result(1.0, Some(2.0), Some(3.0), &config());
        for m in &result.metrics {
            if m.name != "memory_pressure_avg60" {
                assert!(m.threshold_warning.is_none() && m.threshold_critical.is_none());
            }
        }
    }
}
