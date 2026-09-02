//! Pressure Stall Information (PSI) — the modern, honest signal that the
//! machine is *struggling*, well before a raw usage percentage looks alarming.
//! Reads `/proc/pressure/{memory,cpu,io}`, which report two figures per
//! resource: `some`, the share of the window in which *at least one* task was
//! stalled, and `full`, the share in which *every* runnable task was.
//!
//! Only memory `full avg60` carries a threshold. `some` memory pressure is a
//! normal by-product of healthy work — a kernel build or a large file copy
//! reclaims page cache and pushes `some` to 20–30% on a machine in no trouble
//! at all — so alerting on it would have Joséphine interrupt you for doing
//! exactly what the computer is for. `full` only climbs when nothing can make
//! progress, which is the swap death-spiral this check exists to catch.
//!
//! `some` memory, CPU and IO are recorded too (for history and `doctor`), but
//! carry no thresholds and never raise an alert. Degrades to "unavailable"
//! where PSI isn't exposed (older kernels, or `CONFIG_PSI` disabled).

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
        // `full` is what we judge on, so without it there is nothing to say.
        let Some(memory) = read_psi("memory") else {
            return Ok(unavailable());
        };
        let Some(memory_full) = memory.full else {
            return Ok(unavailable());
        };
        let cpu = read_psi("cpu").map(|psi| psi.some);
        let io = read_psi("io").map(|psi| psi.some);
        Ok(build_result(
            memory_full,
            memory.some,
            cpu,
            io,
            &self.config,
        ))
    }
}

fn build_result(
    memory_full: f64,
    memory_some: f64,
    cpu: Option<f64>,
    io: Option<f64>,
    config: &PressureCheckConfig,
) -> CheckResult {
    let status_value = match i18n::lang() {
        Lang::En => format!("{memory_full:.0}% stalled (60s)"),
        Lang::Fr => format!("{memory_full:.0}% à l'arrêt (60s)"),
    };

    let mut details = vec![match i18n::lang() {
        Lang::En => format!(
            "Memory stall: {memory_full:.1}% of the last minute with everything waiting ({memory_some:.1}% with something waiting)."
        ),
        Lang::Fr => format!(
            "Blocage mémoire : {memory_full:.1}% de la dernière minute tout à l'arrêt ({memory_some:.1}% avec au moins une tâche en attente)."
        ),
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

    let mut metrics = vec![
        Metric {
            name: "memory_pressure_full_avg60".into(),
            value: memory_full,
            unit: "%".into(),
            threshold_warning: Some(config.warning),
            threshold_critical: Some(config.critical),
        },
        // `some` is kept for history and for the detail line, but never alerts:
        // healthy work (a build, a big copy) drives it to 20–30% routinely.
        Metric {
            name: "memory_pressure_avg60".into(),
            value: memory_some,
            unit: "%".into(),
            threshold_warning: None,
            threshold_critical: None,
        },
    ];
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

/// One PSI resource's 60-second averages.
struct Psi {
    /// Share of the window with *at least one* task stalled.
    some: f64,
    /// Share of the window with *every* runnable task stalled. Absent on some
    /// resources and older kernels (notably `cpu` before 5.13).
    full: Option<f64>,
}

/// Read the 60-second averages for a PSI resource (`memory`, `cpu`, `io`).
fn read_psi(resource: &str) -> Option<Psi> {
    let content = std::fs::read_to_string(format!("/proc/pressure/{resource}")).ok()?;
    parse_psi(&content)
}

/// Parse a PSI file. `some` is required (a file without it isn't PSI output);
/// `full` is optional.
fn parse_psi(content: &str) -> Option<Psi> {
    Some(Psi {
        some: parse_avg60(content, "some")?,
        full: parse_avg60(content, "full"),
    })
}

/// The `avg60` field of the named PSI line (`some` or `full`).
fn parse_avg60(content: &str, kind: &str) -> Option<f64> {
    for line in content.lines() {
        let Some(rest) = line.trim().strip_prefix(kind) else {
            continue;
        };
        let Some(rest) = rest.strip_prefix(' ') else {
            continue;
        };
        for field in rest.split_whitespace() {
            if let Some(value) = field.strip_prefix("avg60=") {
                return value.parse::<f64>().ok();
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
    fn parses_both_lines() {
        let psi = parse_psi(SAMPLE).expect("psi");
        assert_eq!(psi.some, 3.40);
        assert_eq!(psi.full, Some(0.00));
    }

    #[test]
    fn missing_some_line_is_none() {
        assert!(parse_psi("full avg60=1.0 total=0\n").is_none());
    }

    /// `cpu` has no `full` line before 5.13; `some` alone still parses.
    #[test]
    fn a_missing_full_line_is_tolerated() {
        let psi = parse_psi("some avg10=0.03 avg60=0.04 avg300=0.85 total=8675\n").expect("psi");
        assert_eq!(psi.some, 0.04);
        assert_eq!(psi.full, None);
    }

    #[test]
    fn calm_memory_is_info() {
        let result = build_result(0.0, 0.5, Some(0.1), Some(0.0), &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Info);
    }

    #[test]
    fn sustained_full_stall_is_critical() {
        let result = build_result(30.0, 80.0, Some(1.0), None, &config());
        assert_eq!(result.worst_severity(), crate::check::Severity::Critique);
    }

    /// The regression this guards: a kernel build or a large copy reclaims page
    /// cache and pushes `some` to 20–30% while nothing is actually wrong.
    /// Alerting on that would interrupt the user for working the machine.
    #[test]
    fn a_busy_build_does_not_raise_an_alert() {
        for some in [20.0, 30.0, 45.0] {
            let result = build_result(0.0, some, Some(35.0), Some(25.0), &config());
            assert_eq!(
                result.worst_severity(),
                crate::check::Severity::Info,
                "some={some} must stay quiet while full is 0"
            );
        }
    }

    #[test]
    fn only_the_full_memory_metric_carries_thresholds() {
        let result = build_result(1.0, 2.0, Some(3.0), Some(4.0), &config());
        for m in &result.metrics {
            if m.name == "memory_pressure_full_avg60" {
                assert!(m.threshold_warning.is_some() && m.threshold_critical.is_some());
            } else {
                assert!(
                    m.threshold_warning.is_none() && m.threshold_critical.is_none(),
                    "{} must not alert",
                    m.name
                );
            }
        }
    }
}
