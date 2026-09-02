use colored::Colorize;
use josephine_core::check::{CheckResult, Metric, Severity};
use josephine_core::checks::interval_for_check;
use josephine_core::config::Config;
use josephine_core::i18n;
use josephine_core::remedy::{self, Remedy};
use josephine_core::voice;

use super::bars::{BAR_WIDTH, bar_plain};
use super::style::{check_label, format_metric_value, metric_scale, primary_metric};

pub fn print_doctor(results: &[CheckResult], config: &Config, verbose: bool, foresight: &[String]) {
    let worst = results
        .iter()
        .map(CheckResult::worst_severity)
        .max()
        .unwrap_or(Severity::Info);
    // Lead with the diagnosis (the verdict), then the count, then the exam.
    super::style::sober_header(
        Some(i18n::t("diagnostic", "diagnostic")),
        Some(voice::doctor_verdict(worst)),
    );
    let summary = summary_line(results);
    if super::style::is_tty() {
        println!(" {}", summary.dimmed());
    } else {
        println!(" {summary}");
    }
    println!();
    for result in results {
        print_check_block(result, config, verbose);
    }
    print_foresight_section(foresight);
    print_todo_section(results);
    println!();
    super::style::sober_footer(footer_hint(verbose));
}

fn summary_line(results: &[CheckResult]) -> String {
    let total = results.len();
    let n = results
        .iter()
        .filter(|r| r.worst_severity() != Severity::Info)
        .count();
    match i18n::lang() {
        i18n::Lang::En => format!("{total} checks · {n} to look at"),
        i18n::Lang::Fr => format!("{total} contrôles · {n} à regarder"),
    }
}

fn state_word(severity: Severity) -> String {
    let w = match severity {
        Severity::Info => i18n::t("ok", "ok"),
        Severity::Attention => i18n::t("attention", "attention"),
        Severity::Critique => i18n::t("critical", "critique"),
    };
    super::style::severity_paint(w, severity)
}

fn print_check_block(result: &CheckResult, config: &Config, verbose: bool) {
    let severity = result.worst_severity();
    let glyph = super::style::status_glyph(severity);
    let label = check_label(&result.check_name);
    println!(" {glyph}  {label} · {}", state_word(severity));
    for line in detail_lines(result, config, verbose) {
        println!("    {line}");
    }
}

/// The forward-looking section: where the slow trends are heading. Prints
/// nothing unless a forecast crossed its horizon, so a quiet machine stays
/// quiet — Joséphine only speaks when the trend is real and near.
fn print_foresight_section(foresight: &[String]) {
    if foresight.is_empty() {
        return;
    }
    println!();
    println!("  {}", i18n::t("Looking ahead", "Prévoyance"));
    for line in foresight {
        println!("    {line}");
    }
}

/// The grouped closing section: every action left to take, most severe first.
///
/// Prints nothing when there are no remedies to show — a healthy machine
/// gets no to-do list, and so does one where every failing check happens to
/// have none.
fn print_todo_section(results: &[CheckResult]) {
    let mut failing: Vec<&CheckResult> = results
        .iter()
        .filter(|r| r.worst_severity() != Severity::Info)
        .collect();
    // Stable sort: Critique before Attention, insertion order kept within a band.
    failing.sort_by_key(|r| std::cmp::Reverse(r.worst_severity()));

    let remedies: Vec<Remedy> = failing.into_iter().flat_map(remedy::for_result).collect();
    if remedies.is_empty() {
        return;
    }

    println!();
    println!(
        "  {}",
        i18n::t("What's left to do", "Ce qu'il reste à faire")
    );
    for (i, entry) in remedies.iter().enumerate() {
        println!("    {}. {}", i + 1, entry.action);
        if let Some(hint) = &entry.hint {
            println!("       {hint}");
        }
    }
    println!();
    println!(" {}", voice::remedy_hands_off());
}

fn footer_hint(verbose: bool) -> &'static str {
    if verbose {
        i18n::t(
            "Condensed view: `josephine doctor` (without --verbose).",
            "Vue condensée : `josephine doctor` (sans --verbose).",
        )
    } else {
        i18n::t(
            "See everything (thresholds, processes, intervals): `josephine doctor --verbose`.",
            "Tout voir (seuils, processus, intervalles) : `josephine doctor --verbose`.",
        )
    }
}

/// Compose the multi-line "Mesure & détails" cell for one check.
fn detail_lines(result: &CheckResult, config: &Config, verbose: bool) -> Vec<String> {
    let mut lines = Vec::new();

    // Thresholded metrics as bar lines — primary first, then the rest (swap, …).
    let primary = primary_metric(result);
    let primary_name = primary.map(|m| m.name.as_str());
    let mut ordered: Vec<&Metric> = Vec::new();
    if let Some(p) = primary {
        ordered.push(p);
    }
    for metric in &result.metrics {
        if metric.threshold_warning.is_some() && Some(metric.name.as_str()) != primary_name {
            ordered.push(metric);
        }
    }

    for (i, metric) in ordered.iter().enumerate() {
        let bar = bar_plain(metric.value, metric_scale(metric), BAR_WIDTH);
        let value = format_metric_value(metric);
        if i == 0 {
            lines.push(format!("{bar}  {value}"));
        } else {
            lines.push(format!("{}  {bar}  {value}", metric_label(&metric.name)));
        }
        if verbose && let (Some(w), Some(c)) = (metric.threshold_warning, metric.threshold_critical)
        {
            lines.push(format!(
                "  {} {} · {} {}",
                i18n::t("warning", "seuil alerte"),
                fmt_threshold(w, &metric.unit),
                i18n::t("critical", "critique"),
                fmt_threshold(c, &metric.unit)
            ));
        }
    }

    // System facts (load, sensors, partitions, failed services…).
    for detail in &result.details {
        lines.push(detail.trim().to_string());
    }

    // Process lists live in `top_processes` for cpu/memory — 3 normally, 10 in verbose.
    if let Some(header) = process_header(&result.check_name) {
        let limit = if verbose { 10 } else { 3 };
        let processes: Vec<&String> = result.top_processes.iter().take(limit).collect();
        if !processes.is_empty() {
            lines.push(header.to_string());
            for process in processes {
                lines.push(format!("• {process}"));
            }
        }
    }

    if verbose {
        let secs = interval_for_check(&result.check_name, &config.checks);
        lines.push(format!(
            "{} {}",
            i18n::t("Collected", "Collecte"),
            human_interval(secs)
        ));
    }

    lines
}

fn process_header(check_name: &str) -> Option<&'static str> {
    match check_name {
        "cpu" => Some(i18n::t("Busiest processes:", "Processus les plus actifs :")),
        "memory" => Some(i18n::t(
            "Hungriest processes:",
            "Processus les plus gourmands :",
        )),
        _ => None,
    }
}

fn fmt_threshold(value: f64, unit: &str) -> String {
    match unit {
        "%" => format!("{value:.0} %"),
        "°C" => format!("{value:.0} °C"),
        "ms" => format!("{value:.0} ms"),
        _ => format!("{value:.0}"),
    }
}

fn human_interval(secs: u64) -> String {
    let every = i18n::t("every", "toutes les");
    if secs >= 3600 && secs % 3600 == 0 {
        format!("{every} {} h", secs / 3600)
    } else if secs >= 60 && secs % 60 == 0 {
        format!("{every} {} min", secs / 60)
    } else {
        format!("{every} {secs} s")
    }
}

fn metric_label(name: &str) -> String {
    match name {
        "usage_percent" => i18n::t("Usage", "Utilisation").into(),
        "swap_percent" => "Swap".into(),
        "usage_percent_worst" => i18n::t("Disk (max)", "Disque (max)").into(),
        "temp_max_celsius" => i18n::t("Max temperature", "Température max").into(),
        "failed_units" => i18n::t("Failed services", "Services en échec").into(),
        "max_restarts" => i18n::t("Max restarts", "Redémarrages max").into(),
        "updates_available" => i18n::t("Updates", "Mises à jour").into(),
        "gateway_latency_ms" => i18n::t("Gateway latency", "Latence passerelle").into(),
        "charge_percent" => i18n::t("Charge", "Charge").into(),
        "battery_depletion_percent" => i18n::t("Depletion", "Décharge").into(),
        "inode_usage_percent_worst" => "Inodes (max)".into(),
        "smart_failing" => i18n::t("Failing disks", "Disques en échec").into(),
        "kernel_incidents" => i18n::t("Kernel incidents", "Incidents noyau").into(),
        "readonly_mounts" => i18n::t("Read-only mounts", "Montages en lecture seule").into(),
        "clock_unsynced" => i18n::t("Clock unsynced", "Horloge désynchronisée").into(),
        "failed_auths" => i18n::t("Failed logins", "Connexions échouées").into(),
        other => other.replace('_', " "),
    }
}
