use josephine_core::check::{CheckResult, Severity};
use josephine_core::i18n;
use josephine_core::voice;

use super::style::{format_metric_value, primary_metric};

pub fn print_status_table(results: &[CheckResult]) {
    super::style::sober_header(None, Some(voice::status_tagline()));
    let rows = build_rows(results);
    let label_w = rows
        .iter()
        .map(|r| r.label.chars().count())
        .max()
        .unwrap_or(0)
        + 2;
    for row in &rows {
        print_row(row, label_w);
    }
    print_footer_line(results);
}

// ---------------------------------------------------------------------------
// Check rows
// ---------------------------------------------------------------------------

struct Row {
    label: String,
    value: String,
    severity: Severity,
}

fn build_rows(results: &[CheckResult]) -> Vec<Row> {
    let mut rows = Vec::new();
    for result in results {
        rows.push(check_row(result));
        // The system load lives on the CPU check; surface it as its own line.
        if result.check_name == "cpu"
            && let Some(load_row) = load_row()
        {
            rows.push(load_row);
        }
    }
    rows
}

fn check_row(result: &CheckResult) -> Row {
    let label = super::style::check_label(&result.check_name).to_string();
    let value = result
        .status_value
        .clone()
        .or_else(|| primary_metric(result).map(format_metric_value))
        .unwrap_or_else(|| "—".to_string());

    Row {
        label,
        value,
        severity: result.worst_severity(),
    }
}

fn load_row() -> Option<Row> {
    let (one, five, fifteen) = read_loadavg()?;
    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1) as f64;
    let ratio = one / cores.max(1.0);
    let severity = if ratio >= 2.0 {
        Severity::Critique
    } else if ratio >= 1.0 {
        Severity::Attention
    } else {
        Severity::Info
    };

    Some(Row {
        label: i18n::t("Load", "Charge").to_string(),
        value: format!("{one:.2} · {five:.2} · {fifteen:.2}"),
        severity,
    })
}

fn read_loadavg() -> Option<(f64, f64, f64)> {
    let content = std::fs::read_to_string("/proc/loadavg").ok()?;
    let mut it = content.split_whitespace();
    let one = it.next()?.parse().ok()?;
    let five = it.next()?.parse().ok()?;
    let fifteen = it.next()?.parse().ok()?;
    Some((one, five, fifteen))
}

fn print_row(row: &Row, label_w: usize) {
    let glyph = super::style::status_glyph(row.severity);
    let label = pad(&row.label, label_w);
    let value = super::style::severity_paint(&row.value, row.severity);
    println!(" {glyph}  {label}{value}");
}

// ---------------------------------------------------------------------------
// Footer
// ---------------------------------------------------------------------------

fn footer_message(count: usize) -> String {
    use josephine_core::i18n::{self, Lang};
    if count == 0 {
        return voice::all_clear().to_string();
    }
    match i18n::lang() {
        Lang::En => format!(
            "{count} thing{} to look at → josephine doctor",
            if count > 1 { "s" } else { "" }
        ),
        Lang::Fr => format!(
            "{count} point{} à regarder → josephine doctor",
            if count > 1 { "s" } else { "" }
        ),
    }
}

fn print_footer_line(results: &[CheckResult]) {
    let count = results
        .iter()
        .filter(|r| r.worst_severity() != Severity::Info)
        .count();
    super::style::sober_footer(&footer_message(count));
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Pad a string to `width` display columns (approximated by char count).
fn pad(s: &str, width: usize) -> String {
    let len = s.chars().count();
    if len >= width {
        s.to_string()
    } else {
        format!("{s}{}", " ".repeat(width - len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn footer_message_pluralizes() {
        use josephine_core::i18n::{self, Lang};
        // A different test binary than josephine-core's own (this crate's
        // integration tests spawn a subprocess and can't race this), but
        // still shares this crate's test binary with any other test here
        // that touches the language — guard for consistency with core.
        let _guard = i18n::lock_for_test();
        let prev = i18n::lang();
        i18n::set_lang(Lang::En);
        // The zero-issue line is varied (voice::all_clear); just assert it speaks up.
        assert!(!footer_message(0).is_empty());
        assert!(footer_message(1).starts_with("1 thing to look at"));
        assert!(footer_message(3).starts_with("3 things to look at"));
        i18n::set_lang(prev);
    }
}
