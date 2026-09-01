use std::io::{self, IsTerminal, Write};

use anyhow::Result;
use chrono::Local;
use colored::Color;
use colored::Colorize;
use josephine_core::check::{Metric, Severity, metric_severity};
use josephine_core::i18n::{self, Lang};
use josephine_core::paths::Paths;

pub fn is_tty() -> bool {
    std::io::stdout().is_terminal()
}

/// Width of the header rule and the clock's right edge.
pub const HEADER_WIDTH: usize = 54;

/// Soft indigo/violet — Joséphine's discreet stellar accent.
const ACCENT: (u8, u8, u8) = (150, 130, 220);

pub fn accent(s: &str) -> String {
    s.truecolor(ACCENT.0, ACCENT.1, ACCENT.2).to_string()
}

fn severity_color(severity: Severity) -> Color {
    match severity {
        Severity::Info => Color::Green,
        Severity::Attention => Color::Yellow,
        Severity::Critique => Color::Red,
    }
}

pub fn severity_paint(s: &str, severity: Severity) -> String {
    s.color(severity_color(severity)).to_string()
}

/// Status glyph carrying severity by shape *and* colour. Off a terminal it
/// degrades to an ASCII tag so pipes and logs stay readable.
pub fn status_glyph(severity: Severity) -> String {
    let (glyph, plain) = match severity {
        Severity::Info => ("●", "[ok]"),
        Severity::Attention => ("▲", "[!]"),
        Severity::Critique => ("✕", "[x]"),
    };
    if is_tty() {
        severity_paint(glyph, severity)
    } else {
        // Uniform width so non-TTY rows stay column-aligned across severities.
        format!("{plain:<4}")
    }
}

/// Compact status token for the one-line output — same shape+colour glyph as
/// [`status_glyph`], but without the column padding, so it reads cleanly in a
/// status bar. Off a terminal it degrades to a bare `[ok]`/`[!]`/`[x]` tag.
pub fn oneline_glyph(severity: Severity) -> String {
    let (glyph, plain) = match severity {
        Severity::Info => ("●", "[ok]"),
        Severity::Attention => ("▲", "[!]"),
        Severity::Critique => ("✕", "[x]"),
    };
    if is_tty() {
        severity_paint(glyph, severity)
    } else {
        plain.to_string()
    }
}

/// Load a user banner from `<config dir>/banner.txt`, if present and non-empty.
///
/// Private helper for `sober_header`, which prints the banner (if any) above
/// the header.
fn custom_banner() -> Option<Vec<String>> {
    let paths = Paths::new().ok()?;
    let dir = paths.config.parent()?;
    let content = std::fs::read_to_string(dir.join("banner.txt")).ok()?;
    if content.trim().is_empty() {
        return None;
    }
    Some(content.lines().map(str::to_string).collect())
}

/// Print each banner line tinted from amber (top) to violet (bottom).
///
/// Private helper for `sober_header`, used to render the optional
/// `banner.txt` above the header.
fn print_banner_gradient(lines: &[String]) {
    let n = lines.len();
    for (i, line) in lines.iter().enumerate() {
        let t = if n <= 1 {
            0.0
        } else {
            i as f64 / (n - 1) as f64
        };
        let r = lerp(224.0, 158.0, t);
        let g = lerp(164.0, 128.0, t);
        let b = lerp(88.0, 210.0, t);
        println!("{}", line.truecolor(r, g, b));
    }
}

/// Linear interpolation between `a` and `b` at `t` (`0.0..=1.0`), used by
/// `print_banner_gradient` to compute each line's tint.
fn lerp(a: f64, b: f64, t: f64) -> u8 {
    (a + (b - a) * t).round() as u8
}

/// Print the sober header: an optional `banner.txt` on top, then
/// `✦ Joséphine[ · suffix]` with a right-aligned clock, an optional dimmed
/// tagline, and a thin rule.
pub fn sober_header(suffix: Option<&str>, tagline: Option<&str>) {
    println!();
    if let Some(banner) = custom_banner() {
        print_banner_gradient(&banner);
        println!();
    }
    let clock = Local::now().format("%H:%M").to_string();
    let mut title = String::from("✦ Joséphine");
    if let Some(s) = suffix {
        title.push_str(&format!(" · {s}"));
    }
    if is_tty() {
        let pad = HEADER_WIDTH.saturating_sub(title.chars().count() + clock.chars().count());
        println!(
            "{}{}{}",
            accent(&title),
            " ".repeat(pad.max(1)),
            clock.dimmed()
        );
    } else {
        println!("{title}  {clock}");
    }
    if let Some(t) = tagline {
        println!(
            "{}",
            if is_tty() {
                t.dimmed().to_string()
            } else {
                t.to_string()
            }
        );
    }
    println!("{}", "─".repeat(HEADER_WIDTH).dimmed());
}

/// Ask a yes/no question on the terminal. Returns `false` on a non-interactive
/// stdin (so nothing destructive ever runs unattended without `--yes`).
pub fn confirm(question: &str) -> Result<bool> {
    if !is_tty() {
        return Ok(false);
    }
    print!("{question}{}", i18n::t(" [y/N] ", " [o/N] "));
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    Ok(matches!(
        answer.trim().to_lowercase().as_str(),
        "o" | "oui" | "y" | "yes"
    ))
}

/// Render a compact unicode sparkline (`▁▂▃▅▇`) from a series of values,
/// scaled between the series' own min and max.
pub fn sparkline(values: &[f64]) -> String {
    const TICKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    if values.is_empty() {
        return "—".to_string();
    }
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span = max - min;
    values
        .iter()
        .map(|&v| {
            let idx = if span <= f64::EPSILON {
                TICKS.len() / 2
            } else {
                (((v - min) / span) * (TICKS.len() - 1) as f64).round() as usize
            };
            TICKS[idx.min(TICKS.len() - 1)]
        })
        .collect()
}

pub fn check_label(name: &str) -> &'static str {
    match name {
        "cpu" => "CPU",
        "memory" => i18n::t("Memory", "Mémoire"),
        "disk" => i18n::t("Disk", "Disque"),
        "temperature" => i18n::t("Temperature", "Température"),
        "systemd" => i18n::t("systemd services", "Services systemd"),
        "updates" => i18n::t("Updates", "Mises à jour"),
        "network" => i18n::t("Network", "Réseau"),
        "battery" => i18n::t("Battery", "Batterie"),
        "inode" => "Inodes",
        "smart" => i18n::t("Disk health", "Santé disque"),
        "kernel" => i18n::t("Kernel", "Noyau"),
        "filesystem" => i18n::t("Filesystem", "Système de fichiers"),
        "timesync" => i18n::t("Clock", "Horloge"),
        "security" => i18n::t("Security", "Sécurité"),
        "reboot" => i18n::t("Reboot", "Redémarrage"),
        "pressure" => i18n::t("Pressure", "Pression"),
        _ => i18n::t("System", "Système"),
    }
}

/// Print the sober closing rule + a one-space-indented, dimmed footer
/// message. Shared bottom treatment for `status` and `doctor`.
pub fn sober_footer(msg: &str) {
    println!("{}", "─".repeat(HEADER_WIDTH).dimmed());
    if is_tty() {
        println!(" {}", msg.dimmed());
    } else {
        println!(" {msg}");
    }
}

pub fn primary_metric(result: &josephine_core::check::CheckResult) -> Option<&Metric> {
    match result.check_name.as_str() {
        "cpu" => result.metrics.iter().find(|m| m.name == "usage_percent"),
        "memory" => result.metrics.iter().find(|m| m.name == "usage_percent"),
        "disk" => result
            .metrics
            .iter()
            .find(|m| m.name == "usage_percent_worst"),
        "temperature" => result.metrics.iter().find(|m| m.name == "temp_max_celsius"),
        "systemd" => result
            .metrics
            .iter()
            .max_by_key(|m| (metric_severity(m), m.name == "failed_units")),
        "network" => result
            .metrics
            .iter()
            .find(|m| m.name == "gateway_latency_ms"),
        "battery" => result.metrics.iter().find(|m| m.name == "charge_percent"),
        "inode" => result
            .metrics
            .iter()
            .find(|m| m.name == "inode_usage_percent_worst"),
        "smart" => result.metrics.iter().find(|m| m.name == "smart_failing"),
        "kernel" => result.metrics.iter().find(|m| m.name == "kernel_incidents"),
        "filesystem" => result.metrics.iter().find(|m| m.name == "readonly_mounts"),
        "timesync" => result.metrics.iter().find(|m| m.name == "clock_unsynced"),
        "security" => result.metrics.iter().find(|m| m.name == "failed_auths"),
        "reboot" => result.metrics.iter().find(|m| m.name == "reboot_required"),
        "pressure" => result
            .metrics
            .iter()
            .find(|m| m.name == "memory_pressure_avg60"),
        _ => result.metrics.first(),
    }
}

pub fn metric_scale(metric: &Metric) -> f64 {
    metric
        .threshold_critical
        .or(metric.threshold_warning)
        .unwrap_or(100.0)
        .max(1.0)
}

pub fn format_metric_value(metric: &Metric) -> String {
    match metric.unit.as_str() {
        "%" => format!("{:.1} %", metric.value),
        "°C" => format!("{:.1} °C", metric.value),
        "services" => {
            let n = metric.value as u64;
            if n <= 1 {
                format!("{n} service")
            } else {
                format!("{n} services")
            }
        }
        "restarts" => {
            let n = metric.value as u64;
            match (i18n::lang(), n) {
                (Lang::En, 0..=1) => format!("{n} restart"),
                (Lang::En, _) => format!("{n} restarts"),
                (Lang::Fr, 0..=1) => format!("{n} redémarrage"),
                (Lang::Fr, _) => format!("{n} redémarrages"),
            }
        }
        "updates" => {
            let n = metric.value as u64;
            match (i18n::lang(), n) {
                (Lang::En, 0) => "up to date".to_string(),
                (Lang::En, 1) => "1 update".to_string(),
                (Lang::En, _) => format!("{n} updates"),
                (Lang::Fr, 0) => "à jour".to_string(),
                (Lang::Fr, 1) => "1 mise à jour".to_string(),
                (Lang::Fr, _) => format!("{n} mises à jour"),
            }
        }
        "ms" => format!("{:.0} ms", metric.value),
        "disks" => {
            let n = metric.value as u64;
            match i18n::lang() {
                Lang::En => format!("{n} disk(s)"),
                Lang::Fr => format!("{n} disque(s)"),
            }
        }
        "events" => {
            let n = metric.value as u64;
            format!("{n} incident(s)")
        }
        "mounts" => {
            let n = metric.value as u64;
            match (i18n::lang(), n) {
                (Lang::En, 0..=1) => format!("{n} mount"),
                (Lang::En, _) => format!("{n} mounts"),
                (Lang::Fr, 0..=1) => format!("{n} montage"),
                (Lang::Fr, _) => format!("{n} montages"),
            }
        }
        _ => format!("{:.1} {}", metric.value, metric.unit),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparkline_empty_is_dash() {
        assert_eq!(sparkline(&[]), "—");
    }

    #[test]
    fn sparkline_flat_series_is_uniform() {
        let line = sparkline(&[5.0, 5.0, 5.0]);
        assert_eq!(line.chars().count(), 3);
        // A flat series maps every point to the middle tick.
        assert!(line.chars().all(|c| c == '▅'));
    }

    #[test]
    fn sparkline_rises_with_values() {
        let line = sparkline(&[0.0, 50.0, 100.0]);
        let ticks: Vec<char> = line.chars().collect();
        assert_eq!(ticks.len(), 3);
        assert_eq!(ticks[0], '▁');
        assert_eq!(ticks[2], '█');
        assert!(ticks[0] < ticks[1] && ticks[1] < ticks[2]);
    }

    #[test]
    fn status_glyph_is_ascii_off_tty() {
        use josephine_core::check::Severity;
        // Off a terminal (as in `cargo test`) the glyph degrades to an ASCII tag,
        // padded to a uniform width so rows stay column-aligned across severities.
        assert_eq!(status_glyph(Severity::Info), "[ok]");
        assert_eq!(status_glyph(Severity::Attention), "[!] ");
        assert_eq!(status_glyph(Severity::Critique), "[x] ");
        let w = status_glyph(Severity::Info).chars().count();
        assert_eq!(status_glyph(Severity::Attention).chars().count(), w);
        assert_eq!(status_glyph(Severity::Critique).chars().count(), w);
    }
}
