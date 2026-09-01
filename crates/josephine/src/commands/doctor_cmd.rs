use anyhow::Result;
use josephine_core::config::Config;
use josephine_core::forecast::{self, Guards};
use josephine_core::i18n::{self, Lang};
use josephine_core::paths::Paths;
use josephine_core::storage::Storage;

use crate::output::{check_label, print_checks_json, print_doctor, run_checks_with_progress};

pub fn run(verbose: bool, json: bool) -> Result<()> {
    let config = Config::load_default()?;
    let results = run_checks_with_progress(&config)?;
    if json {
        // Foresight is a human-facing, doctor-only heads-up; JSON is unchanged.
        print_checks_json(&results);
        return Ok(());
    }
    let foresight = foresight_lines(&config);
    print_doctor(&results, &config, verbose, &foresight);
    Ok(())
}

/// What Joséphine watches and where it's heading. Each target projects one
/// stored metric toward a limit (a full disk, a worn-out SSD).
struct Target {
    check: &'static str,
    metric: &'static str,
    goal: f64,
    kind: Kind,
}

#[derive(Clone, Copy)]
enum Kind {
    Fill,
    Wear,
}

const TARGETS: &[Target] = &[
    Target {
        check: "disk",
        metric: "usage_percent_worst",
        goal: 100.0,
        kind: Kind::Fill,
    },
    Target {
        check: "inode",
        metric: "inode_usage_percent_worst",
        goal: 100.0,
        kind: Kind::Fill,
    },
    Target {
        check: "memory",
        metric: "usage_percent",
        goal: 100.0,
        kind: Kind::Fill,
    },
    Target {
        check: "smart",
        metric: "smart_wear_percent",
        goal: 100.0,
        kind: Kind::Wear,
    },
];

/// Project each curated target from the stored history and render the ones that
/// cross their horizon. Reads only; degrades to nothing when there's no history
/// yet (a fresh install) or the daemon has never run.
fn foresight_lines(config: &Config) -> Vec<String> {
    if !config.forecast.enabled {
        return Vec::new();
    }
    let Ok(paths) = Paths::new() else {
        return Vec::new();
    };
    let Ok(storage) = Storage::open(&paths) else {
        return Vec::new();
    };
    let guards = Guards {
        min_samples: config.forecast.min_samples,
        min_fit: config.forecast.min_fit,
        horizon_days: config.forecast.horizon_days,
    };

    let mut lines = Vec::new();
    for target in TARGETS {
        // SMART (and thus its wear metric) is opt-in.
        if target.check == "smart" && !config.checks.smart.enabled {
            continue;
        }
        let Ok(points) =
            storage.metric_series_since(target.check, target.metric, config.history.retention_days)
        else {
            continue;
        };
        if let Some(forecast) = forecast::project(&points, target.goal, &guards) {
            lines.push(render_forecast(target, &forecast));
        }
    }
    lines
}

fn render_forecast(target: &Target, forecast: &forecast::Forecast) -> String {
    let label = check_label(target.check);
    let eta = forecast.eta_days.round().max(1.0) as i64;
    let now = forecast.value_now;
    let slope = forecast.slope_per_day;
    let days = days_word(eta);
    match (target.kind, i18n::lang()) {
        (Kind::Fill, Lang::En) => {
            format!("{label}: full in ~{eta} {days} ({now:.0}% now, {slope:+.1}%/day)")
        }
        (Kind::Fill, Lang::Fr) => {
            format!("{label} : plein dans ~{eta} {days} ({now:.0} % aujourd'hui, {slope:+.1} %/j)")
        }
        (Kind::Wear, Lang::En) => {
            format!("{label}: worn out in ~{eta} {days} ({now:.0}% used, {slope:+.2}%/day)")
        }
        (Kind::Wear, Lang::Fr) => {
            format!("{label} : usé dans ~{eta} {days} ({now:.0} % utilisé, {slope:+.2} %/j)")
        }
    }
}

fn days_word(eta: i64) -> &'static str {
    match i18n::lang() {
        Lang::En if eta == 1 => "day",
        Lang::En => "days",
        Lang::Fr if eta == 1 => "jour",
        Lang::Fr => "jours",
    }
}
