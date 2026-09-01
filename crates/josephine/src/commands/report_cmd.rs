//! `josephine report` — a plain-text, dated snapshot of the machine's health,
//! printed to stdout or written to a file for archiving. With `--since`, it
//! appends a digest of what happened over a window (e.g. a weekly summary).

use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Local, Utc};
use josephine_core::check::{CheckResult, Severity};
use josephine_core::config::Config;
use josephine_core::i18n::{self, Lang};
use josephine_core::paths::Paths;
use josephine_core::scheduler::run_all_checks;
use josephine_core::storage::{EventRecord, Storage};

use crate::output::{check_label, format_metric_value, primary_metric, print_checks_json};

/// Cap on the events listed in a digest, so a very noisy window can't produce
/// an unbounded report.
const DIGEST_LIMIT: usize = 200;

pub fn run(output: Option<PathBuf>, json: bool, since: Option<String>) -> Result<()> {
    let config = Config::load_default()?;
    let results = run_all_checks(&config)?;

    // `--json` always prints to stdout; `--output` and `--since` are
    // rendered-text concerns and are ignored when `--json` is set.
    if json {
        print_checks_json(&results);
        return Ok(());
    }

    let digest = match since {
        Some(spec) => Some(build_digest(&spec)?),
        None => None,
    };

    let generated = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let report = render_report(&results, &generated, &hostname(), digest.as_ref());

    match output {
        Some(path) => {
            std::fs::write(&path, &report)
                .with_context(|| format!("writing {}", path.display()))?;
            let done = path.display();
            println!(
                "{}",
                match i18n::lang() {
                    Lang::En => format!("Report saved to {done}."),
                    Lang::Fr => format!("Rapport enregistré dans {done}."),
                }
            );
        }
        None => print!("{report}"),
    }
    Ok(())
}

/// The digest of events over a window: how far back, and what happened.
struct Digest {
    window: String,
    events: Vec<EventRecord>,
}

fn build_digest(spec: &str) -> Result<Digest> {
    let hours = parse_since(spec)?;
    let paths = Paths::new()?;
    let storage = Storage::open(&paths)?;
    let events = storage.events_since(hours, DIGEST_LIMIT)?;
    Ok(Digest {
        window: window_label(hours),
        events,
    })
}

/// Parse a `--since` window like `7d` or `24h` into a count of hours.
fn parse_since(spec: &str) -> Result<i64> {
    let spec = spec.trim();
    let (value, per) = if let Some(days) = spec.strip_suffix(['d', 'D']) {
        (days, 24)
    } else if let Some(hours) = spec.strip_suffix(['h', 'H']) {
        (hours, 1)
    } else {
        bail!(i18n::t(
            "--since expects a window like `7d` or `24h`.",
            "--since attend une fenêtre comme `7d` ou `24h`.",
        ));
    };
    let n: i64 = value
        .trim()
        .parse()
        .ok()
        .filter(|&n| n > 0)
        .with_context(|| {
            i18n::t(
                "--since must be a positive number followed by `d` or `h`.",
                "--since doit être un nombre positif suivi de `d` ou `h`.",
            )
        })?;
    Ok(n * per)
}

fn window_label(hours: i64) -> String {
    if hours % 24 == 0 {
        let days = hours / 24;
        match (i18n::lang(), days) {
            (Lang::En, 1) => "1 day".to_string(),
            (Lang::En, d) => format!("{d} days"),
            (Lang::Fr, 1) => "1 jour".to_string(),
            (Lang::Fr, d) => format!("{d} jours"),
        }
    } else {
        match (i18n::lang(), hours) {
            (Lang::En, 1) => "1 hour".to_string(),
            (Lang::En, h) => format!("{h} hours"),
            (Lang::Fr, 1) => "1 heure".to_string(),
            (Lang::Fr, h) => format!("{h} heures"),
        }
    }
}

fn render_report(
    results: &[CheckResult],
    generated: &str,
    host: &str,
    digest: Option<&Digest>,
) -> String {
    let global = results
        .iter()
        .map(CheckResult::worst_severity)
        .max()
        .unwrap_or(Severity::Info);

    let mut out = String::new();
    out.push_str(i18n::t(
        "Joséphine — system report\n",
        "Joséphine — rapport système\n",
    ));
    out.push_str(&format!("{:<12}: {generated}\n", i18n::t("Date", "Date")));
    out.push_str(&format!("{:<12}: {host}\n", i18n::t("Machine", "Machine")));
    out.push_str(&format!(
        "{:<12}: {}\n",
        i18n::t("Global state", "État global"),
        state_label(global)
    ));
    out.push_str(&"=".repeat(60));
    out.push('\n');

    for result in results {
        let severity = result.worst_severity();
        let value = result
            .status_value
            .clone()
            .or_else(|| primary_metric(result).map(format_metric_value))
            .unwrap_or_else(|| "—".to_string());

        out.push_str(&format!(
            "\n[{}] {} — {}\n",
            state_label(severity),
            check_label(&result.check_name),
            value
        ));
        for detail in &result.details {
            out.push_str(&format!("    {}\n", detail.trim()));
        }
    }

    if let Some(digest) = digest {
        out.push('\n');
        out.push_str(&render_digest(digest));
    }

    out.push_str("\n------------------------------------------------------------\n");
    out.push_str(i18n::t(
        "Generated by Joséphine · 100% local\n",
        "Généré par Joséphine · 100 % local\n",
    ));
    out
}

fn render_digest(digest: &Digest) -> String {
    let mut out = String::new();
    out.push_str(&"=".repeat(60));
    out.push('\n');
    out.push_str(&match i18n::lang() {
        Lang::En => format!("Over the last {}\n", digest.window),
        Lang::Fr => format!("Sur les {} écoulés\n", digest.window),
    });

    if digest.events.is_empty() {
        out.push_str(i18n::t(
            "    A quiet stretch — nothing to report.\n",
            "    Une période calme — rien à signaler.\n",
        ));
        return out;
    }

    out.push_str(&match (i18n::lang(), digest.events.len()) {
        (Lang::En, 1) => "    1 event\n".to_string(),
        (Lang::En, n) => format!("    {n} events\n"),
        (Lang::Fr, 1) => "    1 événement\n".to_string(),
        (Lang::Fr, n) => format!("    {n} événements\n"),
    });
    for event in &digest.events {
        let when = to_local(&event.created_at);
        out.push_str(&format!(
            "    • [{}] {} — {when}\n",
            transition_label(&event.to_state),
            check_label(&event.check_name),
        ));
    }
    out
}

/// Human label for a transition target state, warm and non-alarmist.
fn transition_label(to_state: &str) -> &'static str {
    match to_state {
        "RECOVERED" => i18n::t("resolved", "résolu"),
        "CRITICAL" => i18n::t("critical", "critique"),
        "WARNING" => i18n::t("attention", "attention"),
        _ => i18n::t("note", "note"),
    }
}

fn to_local(utc: &DateTime<Utc>) -> String {
    utc.with_timezone(&Local)
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

fn state_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Info => "OK      ",
        Severity::Attention => i18n::t("WARNING ", "ATTENTION"),
        Severity::Critique => i18n::t("CRITICAL", "CRITIQUE "),
    }
}

fn hostname() -> String {
    std::fs::read_to_string("/proc/sys/kernel/hostname")
        .map(|s| s.trim().to_string())
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "machine".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_since_accepts_days_and_hours() {
        assert_eq!(parse_since("7d").unwrap(), 168);
        assert_eq!(parse_since("24h").unwrap(), 24);
        assert_eq!(parse_since(" 3D ").unwrap(), 72);
    }

    #[test]
    fn parse_since_rejects_garbage() {
        assert!(parse_since("7").is_err());
        assert!(parse_since("d").is_err());
        assert!(parse_since("0d").is_err());
        assert!(parse_since("-2h").is_err());
        assert!(parse_since("week").is_err());
    }
}
