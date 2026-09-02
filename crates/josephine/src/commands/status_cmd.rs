use anyhow::Result;
use josephine_core::check::Severity;
use josephine_core::config::Config;

use crate::output::{
    print_checks_json, print_status_oneline, print_status_table, run_checks_with_progress,
    worst_severity,
};

/// Run the checks and render them. Returns the worst severity seen, which the
/// caller maps to the process exit code (ok = 0, attention = 1, critical = 2).
pub fn run(json: bool, oneline: bool) -> Result<Severity> {
    let config = Config::load_default()?;
    let results = run_checks_with_progress(&config)?;
    let worst = worst_severity(&results);
    if json {
        print_checks_json(&results);
    } else if oneline {
        print_status_oneline(&results);
    } else {
        print_status_table(&results);
    }
    Ok(worst)
}
