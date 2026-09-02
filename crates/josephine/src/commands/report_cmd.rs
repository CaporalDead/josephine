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
    window: Window,
    events: Vec<EventRecord>,
    /// The window held more events than [`DIGEST_LIMIT`]; `events` keeps the
    /// most recent ones. The count line has to say so, or it reads as a total.
    truncated: bool,
}

/// How far back a digest looks, kept as a count and a unit rather than a
/// rendered label: French has to agree in gender and number with the sentence
/// it lands in, which a finished "3 heures" string cannot survive.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Window {
    Days(i64),
    Hours(i64),
}

fn build_digest(spec: &str) -> Result<Digest> {
    let hours = parse_since(spec)?;
    let paths = Paths::new()?;
    let storage = Storage::open(&paths)?;
    // Ask for one more than we show: if it comes back, the window was busier
    // than the cap and the digest is an excerpt, not a tally.
    let mut events = storage.events_since(hours, DIGEST_LIMIT + 1)?;
    let truncated = events.len() > DIGEST_LIMIT;
    events.truncate(DIGEST_LIMIT);
    Ok(Digest {
        window: window(hours),
        events,
        truncated,
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

fn window(hours: i64) -> Window {
    if hours % 24 == 0 {
        Window::Days(hours / 24)
    } else {
        Window::Hours(hours)
    }
}

/// The digest heading, written out whole in each language rather than pasted
/// together from a window label — "Sur les 3 heures écoulés" and "Sur les
/// 1 jour écoulés" are both wrong, and no single label can fix them.
fn digest_heading(window: Window) -> String {
    match (i18n::lang(), window) {
        (Lang::En, Window::Days(1)) => "Over the last day".to_string(),
        (Lang::En, Window::Days(d)) => format!("Over the last {d} days"),
        (Lang::En, Window::Hours(1)) => "Over the last hour".to_string(),
        (Lang::En, Window::Hours(h)) => format!("Over the last {h} hours"),
        (Lang::Fr, Window::Days(1)) => "Sur la dernière journée".to_string(),
        (Lang::Fr, Window::Days(d)) => format!("Sur les {d} derniers jours"),
        (Lang::Fr, Window::Hours(1)) => "Sur la dernière heure".to_string(),
        (Lang::Fr, Window::Hours(h)) => format!("Sur les {h} dernières heures"),
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
    out.push_str(&digest_heading(digest.window));
    out.push('\n');

    if digest.events.is_empty() {
        out.push_str(i18n::t(
            "    A quiet stretch — nothing to report.\n",
            "    Une période calme — rien à signaler.\n",
        ));
        return out;
    }

    // When the window was capped, say what the list actually is — the newest
    // slice — rather than printing the cap as if it were the total.
    out.push_str(
        &match (i18n::lang(), digest.events.len(), digest.truncated) {
            (Lang::En, n, true) => format!("    the {n} most recent events\n"),
            (Lang::En, 1, false) => "    1 event\n".to_string(),
            (Lang::En, n, false) => format!("    {n} events\n"),
            (Lang::Fr, n, true) => format!("    les {n} événements les plus récents\n"),
            (Lang::Fr, 1, false) => "    1 événement\n".to_string(),
            (Lang::Fr, n, false) => format!("    {n} événements\n"),
        },
    );
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
    fn window_splits_whole_days_from_hours() {
        assert_eq!(window(168), Window::Days(7));
        assert_eq!(window(24), Window::Days(1));
        assert_eq!(window(3), Window::Hours(3));
        assert_eq!(window(1), Window::Hours(1));
    }

    /// The heading is a full sentence in each language. French must agree in
    /// gender and number — « heures » is feminine, « jours » masculine — and a
    /// count of one is written out, never "les 1 jour".
    #[test]
    fn digest_heading_agrees_in_both_languages() {
        let _guard = i18n::lock_for_test();
        let prev = i18n::lang();

        i18n::set_lang(Lang::Fr);
        assert_eq!(
            digest_heading(Window::Hours(3)),
            "Sur les 3 dernières heures"
        );
        assert_eq!(digest_heading(Window::Days(7)), "Sur les 7 derniers jours");
        assert_eq!(digest_heading(Window::Days(1)), "Sur la dernière journée");
        assert_eq!(digest_heading(Window::Hours(1)), "Sur la dernière heure");

        i18n::set_lang(Lang::En);
        assert_eq!(digest_heading(Window::Hours(3)), "Over the last 3 hours");
        assert_eq!(digest_heading(Window::Days(7)), "Over the last 7 days");
        assert_eq!(digest_heading(Window::Days(1)), "Over the last day");
        assert_eq!(digest_heading(Window::Hours(1)), "Over the last hour");

        i18n::set_lang(prev);
    }

    /// A capped window must not print the cap as if it were the total.
    #[test]
    fn a_truncated_digest_says_so() {
        let _guard = i18n::lock_for_test();
        let prev = i18n::lang();
        i18n::set_lang(Lang::En);

        let event = EventRecord {
            check_name: "disk".into(),
            metric_name: "usage_percent_worst".into(),
            from_state: "NORMAL".into(),
            to_state: "WARNING".into(),
            value: 90.0,
            message: String::new(),
            created_at: Utc::now(),
        };
        let full = Digest {
            window: Window::Days(7),
            events: vec![event.clone(), event.clone()],
            truncated: false,
        };
        assert!(render_digest(&full).contains("2 events"));

        let capped = Digest {
            events: vec![event.clone(), event],
            truncated: true,
            ..full
        };
        let rendered = render_digest(&capped);
        assert!(
            rendered.contains("the 2 most recent events"),
            "rendered: {rendered}"
        );

        i18n::set_lang(prev);
    }

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
