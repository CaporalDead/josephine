use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::commands::{
    ConfigAction, DaemonAction, NotifyAction, clean_cmd, config_cmd, daemon_cmd, doctor_cmd,
    explain_cmd, history_cmd, notify_cmd, report_cmd, status_cmd, update_cmd,
};

/// Your computer's guardian angel
#[derive(Parser)]
#[command(name = "josephine", about = "Your computer's guardian angel", version)]
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

/// Entry point: parse, dispatch, and map errors to a process exit code.
pub async fn run() -> ExitCode {
    match dispatch().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{} {e}", josephine_core::voice::error_lead());
            ExitCode::from(1)
        }
    }
}

/// Localise the top-level and per-subcommand `--help` text to French.
fn localize_help_fr(command: clap::Command) -> clap::Command {
    command
        .about("L'ange gardien de votre ordinateur")
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

async fn dispatch() -> Result<()> {
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
    let cli = Cli::from_arg_matches(&command.get_matches()).unwrap_or_else(|e| e.exit());

    if cli.daemon_internal {
        return josephine_core::daemon::run_daemon_foreground().await;
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

    match cli.command {
        Some(Commands::Status { json }) => status_cmd::run(json)?,
        Some(Commands::Doctor { verbose, json }) => doctor_cmd::run(verbose, json)?,
        Some(Commands::History) => history_cmd::run()?,
        Some(Commands::Daemon { action }) => daemon_cmd::run(action).await?,
        Some(Commands::Config { action }) => config_cmd::run(action)?,
        Some(Commands::Clean { apply }) => clean_cmd::run(apply)?,
        Some(Commands::Explain { check }) => explain_cmd::run(check.as_deref())?,
        Some(Commands::Report { output, json }) => report_cmd::run(output, json)?,
        Some(Commands::Notify { action }) => notify_cmd::run(action)?,
        Some(Commands::Update { check, yes }) => update_cmd::run(check, yes)?,
        Some(Commands::Completions { shell }) => {
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "josephine",
                &mut std::io::stdout(),
            );
        }
        None => status_cmd::run(false)?,
    }

    Ok(())
}
