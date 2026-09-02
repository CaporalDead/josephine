use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{
    ConfigAction, DaemonAction, NotifyAction, clean_cmd, config_cmd, daemon_cmd, doctor_cmd,
    explain_cmd, history_cmd, notify_cmd, report_cmd, status_cmd, update_cmd,
};

/// Your computer's guardian spirit
#[derive(Parser)]
#[command(name = "josephine", about = "Your computer's guardian spirit", version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Internal mode — spawned by `josephine daemon start`
    #[arg(long = "__daemon__", hide = true)]
    daemon_internal: bool,
}

// `--help` / `--version` follow the configured `language`: `dispatch` post-
// processes this derived tree via `localize_help_fr` when the language is French.
#[derive(Subcommand)]
enum Commands {
    /// Quick summary of your machine's health
    Status {
        /// Print machine-readable JSON to stdout instead of the rendered view
        #[arg(long)]
        json: bool,
        /// Print one compact line for a status bar (Waybar, polybar, tmux, …)
        #[arg(long)]
        oneline: bool,
    },
    /// Full diagnostics
    Doctor {
        /// Detailed report: numeric thresholds, top 10 processes, intervals
        #[arg(short, long)]
        verbose: bool,
        /// Print machine-readable JSON to stdout instead of the rendered view
        #[arg(long)]
        json: bool,
    },
    /// The last 24 hours
    History,
    /// Manage the monitoring daemon
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Report reclaimable disk space (preview by default)
    Clean {
        /// Actually clear the thumbnail cache instead of just previewing
        #[arg(long)]
        apply: bool,
    },
    /// Explain what each check watches, and how to act
    Explain {
        /// One check name (e.g. `cpu`, `disk`); omit to list all
        check: Option<String>,
    },
    /// Dated system report, to the screen or a file
    Report {
        /// Write the report to this file instead of printing it
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Print machine-readable JSON to stdout (implies stdout; ignores `--output`)
        #[arg(long)]
        json: bool,
    },
    /// Desktop notifications
    Notify {
        #[command(subcommand)]
        action: NotifyAction,
    },
    /// Check for and install the latest version of Joséphine
    Update {
        /// Report a new version without installing it
        #[arg(long)]
        check: bool,
        /// Don't wait for confirmation before installing
        #[arg(short = 'y', long = "yes")]
        yes: bool,
    },
    /// Generate shell completions (bash, zsh, fish, …)
    Completions {
        /// Which shell to generate completions for
        shell: clap_complete::Shell,
    },
}

/// The command line was malformed (sysexits `EX_USAGE`).
const EXIT_USAGE: u8 = 64;
/// The command ran but failed (sysexits `EX_SOFTWARE`).
const EXIT_FAILURE: u8 = 70;

/// Entry point: parse, dispatch, and map the outcome to a process exit code.
///
/// `status` (and the bare default) carry the machine's health out through the
/// exit code so Joséphine composes with scripts and status bars: ok = 0,
/// attention = 1, critical = 2.
///
/// Codes `0..=2` are therefore *health*, never failure: a status bar polling
/// `josephine status` must be able to tell "the machine is critical" from
/// "Joséphine could not answer". Anything that stops her from answering lands
/// in a separate band, following sysexits(3): a bad command line exits
/// [`EXIT_USAGE`] (64), and a command that ran and failed exits
/// [`EXIT_FAILURE`] (70).
pub async fn run() -> ExitCode {
    match dispatch().await {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{} {e}", josephine_core::voice::error_lead());
            ExitCode::from(EXIT_FAILURE)
        }
    }
}

/// Map a worst-case severity to a process exit code.
fn severity_code(severity: josephine_core::check::Severity) -> u8 {
    use josephine_core::check::Severity;
    match severity {
        Severity::Info => 0,
        Severity::Attention => 1,
        Severity::Critique => 2,
    }
}

/// Localise the top-level and per-subcommand `--help` text to French.
fn localize_help_fr(command: clap::Command) -> clap::Command {
    command
        .about("L'esprit gardien de votre ordinateur")
        .mut_subcommand("status", |c| {
            c.about("Résumé rapide de la santé de votre machine")
        })
        .mut_subcommand("doctor", |c| c.about("Diagnostic complet"))
        .mut_subcommand("history", |c| c.about("Les dernières 24 heures"))
        .mut_subcommand("daemon", |c| c.about("Gérer le démon de surveillance"))
        .mut_subcommand("config", |c| c.about("Configuration"))
        .mut_subcommand("clean", |c| {
            c.about("Espace disque récupérable (aperçu par défaut)")
        })
        .mut_subcommand("explain", |c| {
            c.about("Expliquer ce que chaque check surveille et comment agir")
        })
        .mut_subcommand("report", |c| {
            c.about("Rapport système daté, à l'écran ou dans un fichier")
        })
        .mut_subcommand("notify", |c| c.about("Notifications desktop"))
        .mut_subcommand("update", |c| {
            c.about("Vérifier et installer la dernière version de Joséphine")
        })
        .mut_subcommand("completions", |c| {
            c.about("Générer les complétions shell (bash, zsh, fish…)")
        })
}

async fn dispatch() -> Result<ExitCode> {
    use clap::{CommandFactory, FromArgMatches};
    use josephine_core::i18n::Lang;

    // Read the configured language WITHOUT creating anything on disk, then build
    // the CLI so `--help` / `--version` render in that language — side-effect-free
    // on a fresh system.
    josephine_core::i18n::set_lang(josephine_core::config::Config::language_or_default());
    let mut command = Cli::command();
    if matches!(josephine_core::i18n::lang(), Lang::Fr) {
        command = localize_help_fr(command);
    }
    // Parse by hand rather than letting clap exit for us: clap's own exit code
    // for a usage error is 2, which is "critical" in our health band. Route it
    // to EXIT_USAGE instead. `--help` / `--version` also arrive here as an
    // `Err`, but they are a success and print to stdout — `use_stderr()` is
    // what tells the two apart.
    let matches = match command.try_get_matches() {
        Ok(matches) => matches,
        Err(e) => {
            let _ = e.print();
            return Ok(ExitCode::from(if e.use_stderr() { EXIT_USAGE } else { 0 }));
        }
    };
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

    if cli.daemon_internal {
        josephine_core::daemon::run_daemon_foreground().await?;
        return Ok(ExitCode::SUCCESS);
    }

    // A real command is running: ensure the config exists (first run) and
    // re-apply its language. `completions` needs neither and must not create
    // files (it generates from the static command tree), so skip it for that.
    if !matches!(
        cli.command,
        Some(Commands::Completions { .. }) | Some(Commands::Explain { .. })
    ) {
        if let Ok(config) = josephine_core::config::Config::load_default() {
            josephine_core::i18n::set_lang(config.language);
        }
    }

    // Only `status` (and the bare default) carry severity out through the exit
    // code; every other command exits 0 unless it errors.
    let code = match cli.command {
        Some(Commands::Status { json, oneline }) => severity_code(status_cmd::run(json, oneline)?),
        Some(Commands::Doctor { verbose, json }) => {
            doctor_cmd::run(verbose, json)?;
            0
        }
        Some(Commands::History) => {
            history_cmd::run()?;
            0
        }
        Some(Commands::Daemon { action }) => {
            daemon_cmd::run(action).await?;
            0
        }
        Some(Commands::Config { action }) => {
            config_cmd::run(action)?;
            0
        }
        Some(Commands::Clean { apply }) => {
            clean_cmd::run(apply)?;
            0
        }
        Some(Commands::Explain { check }) => {
            explain_cmd::run(check.as_deref())?;
            0
        }
        Some(Commands::Report { output, json }) => {
            report_cmd::run(output, json)?;
            0
        }
        Some(Commands::Notify { action }) => {
            notify_cmd::run(action)?;
            0
        }
        Some(Commands::Update { check, yes }) => {
            update_cmd::run(check, yes)?;
            0
        }
        Some(Commands::Completions { shell }) => {
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "josephine",
                &mut std::io::stdout(),
            );
            0
        }
        None => severity_code(status_cmd::run(false, false)?),
    };

    Ok(ExitCode::from(code))
}

#[cfg(test)]
mod tests {
    use super::{EXIT_FAILURE, EXIT_USAGE, severity_code};
    use josephine_core::check::Severity;

    #[test]
    fn severity_maps_to_exit_code() {
        assert_eq!(severity_code(Severity::Info), 0);
        assert_eq!(severity_code(Severity::Attention), 1);
        assert_eq!(severity_code(Severity::Critique), 2);
    }

    /// A script reading the exit code must never mistake a failure for a
    /// health verdict, so the two bands may not overlap.
    #[test]
    fn failure_codes_stay_out_of_the_health_band() {
        let health = [
            severity_code(Severity::Info),
            severity_code(Severity::Attention),
            severity_code(Severity::Critique),
        ];
        for code in [EXIT_USAGE, EXIT_FAILURE] {
            assert!(!health.contains(&code), "{code} collides with a severity");
        }
        assert_ne!(EXIT_USAGE, EXIT_FAILURE);
    }
}
