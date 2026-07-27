//! `josephine update` — check GitHub Releases and, with the user's blessing,
//! download and install the package matching how Joséphine was installed.
//!
//! This is the ONLY place Joséphine touches the network, and only when the user
//! runs the command explicitly — never in the background (the "100 % local" rule).

use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::time::Duration;

use anyhow::{Context, Result, bail};
use josephine_core::i18n::{self, Lang};
use josephine_core::messages;
use josephine_core::update::{self, Asset, InstallPlan, ReleaseInfo, UpdateStatus};

use crate::output::confirm;

const USER_AGENT: &str = concat!("josephine/", env!("CARGO_PKG_VERSION"));

/// Outcome of comparing the downloaded package against its published checksum.
enum Integrity {
    Verified,
    Unverified,
    Mismatch,
}

pub fn run(check_only: bool, assume_yes: bool) -> Result<()> {
    crate::output::sober_header(Some(i18n::t("update", "update")), None);

    let release = match fetch_latest() {
        Ok(release) => release,
        Err(e) => {
            println!(
                "{}",
                i18n::t(
                    "I couldn't reach GitHub to look for news. We'll try again later, no stress.",
                    "Je n'ai pas pu joindre GitHub pour regarder les nouveautés. On réessaiera plus tard, sans stress.",
                )
            );
            println!(
                "{}",
                match i18n::lang() {
                    Lang::En => format!("(detail: {e})"),
                    Lang::Fr => format!("(détail : {e})"),
                }
            );
            return Ok(());
        }
    };

    let current = update::current_version();
    match update::compare(current, &release.version) {
        UpdateStatus::UpToDate => {
            println!("{}", messages::update_up_to_date(current, i18n::lang()))
        }
        UpdateStatus::Ahead => {
            println!(
                "{}",
                messages::update_ahead(current, &release.version, i18n::lang())
            )
        }
        UpdateStatus::Available(version) => {
            println!("{}", messages::update_available(&version, i18n::lang()));
            if !release.html_url.is_empty() {
                println!(
                    "{} {}",
                    i18n::t("Release notes:", "Notes de version :"),
                    release.html_url
                );
            }
            if check_only {
                println!(
                    "{}",
                    i18n::t(
                        "Whenever you like: `josephine update` to install it.",
                        "Quand vous voulez : `josephine update` pour l'installer.",
                    )
                );
            } else {
                apply(&release, assume_yes)?;
            }
        }
    }
    Ok(())
}

fn fetch_latest() -> Result<ReleaseInfo> {
    let body = ureq::get(update::LATEST_RELEASE_URL)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .config()
        .timeout_global(Some(Duration::from_secs(15)))
        .build()
        .call()
        .context("request to the GitHub API")?
        .body_mut()
        .read_to_string()
        .context("reading the GitHub response")?;
    update::parse_release(&body)
}

fn apply(release: &ReleaseInfo, assume_yes: bool) -> Result<()> {
    let channel = update::detect_channel();

    // Channels we can't drive from a downloaded package: just show the command.
    let Some(asset) = release.asset_for(channel) else {
        match update::install_plan(channel, Path::new("")) {
            InstallPlan::Manual(message) => println!("{message}"),
            InstallPlan::Run { .. } => {
                println!(
                    "{} {}",
                    i18n::t("Get the latest version:", "Récupérez la dernière version :"),
                    release.html_url
                );
            }
        }
        return Ok(());
    };

    let ask = match i18n::lang() {
        Lang::En => format!("Download and install “{}”?", asset.name),
        Lang::Fr => format!("Télécharger et installer « {} » ?", asset.name),
    };
    if !assume_yes && !confirm(&ask)? {
        println!(
            "{}",
            i18n::t(
                "Understood, another time then. I'll stay on watch.",
                "Entendu, ce sera pour une autre fois. Je reste de garde.",
            )
        );
        return Ok(());
    }

    let dir = staging_dir()?;
    println!(
        "{}",
        match i18n::lang() {
            Lang::En => format!("Downloading {} …", asset.name),
            Lang::Fr => format!("Téléchargement de {} …", asset.name),
        }
    );
    let package = download(asset, &dir)?;

    match verify(release, asset, &package) {
        Integrity::Mismatch => {
            println!(
                "{}",
                i18n::t(
                    "⚠ The package fingerprint doesn't match the published one. To be safe, I'm installing nothing — check your connection and try again.",
                    "⚠ L'empreinte du paquet ne correspond pas à celle publiée. Par prudence, je n'installe rien — vérifiez votre connexion et réessayez.",
                )
            );
            return Ok(());
        }
        Integrity::Verified => {
            println!(
                "{}",
                i18n::t("Integrity verified ✓", "Intégrité vérifiée ✓")
            )
        }
        Integrity::Unverified => {}
    }

    match update::install_plan(channel, &package) {
        InstallPlan::Run { command, sudo } => {
            println!(
                "{}",
                i18n::t(
                    "Installing the new version — your password may be requested.",
                    "J'installe la nouvelle version — votre mot de passe peut être demandé.",
                )
            );
            let status = run_install(&command, sudo)?;
            if status.success() {
                println!("{}", messages::update_done(&release.version, i18n::lang()));
            } else {
                let pkg = package.display();
                println!(
                    "{}",
                    match i18n::lang() {
                        Lang::En => format!(
                            "The installation didn't complete. The package is ready here: {pkg}"
                        ),
                        Lang::Fr => format!(
                            "L'installation ne s'est pas terminée. Le paquet reste prêt ici : {pkg}"
                        ),
                    }
                );
            }
        }
        InstallPlan::Manual(message) => println!("{message}"),
    }
    Ok(())
}

fn download(asset: &Asset, dir: &Path) -> Result<PathBuf> {
    let dest = dir.join(&asset.name);

    // `into_reader` is unlimited by default, which is what we want for a
    // multi-megabyte package; the checksum below is what guards the content.
    let mut reader = ureq::get(&asset.download_url)
        .header("User-Agent", USER_AGENT)
        .call()
        .context("downloading the package")?
        .into_body()
        .into_reader();
    let mut file =
        std::fs::File::create(&dest).with_context(|| format!("creating {}", dest.display()))?;
    io::copy(&mut reader, &mut file).context("writing the downloaded package")?;

    // World-readable so apt's sandboxed `_apt` user can read the package during
    // install (silences the "unsandboxed" warning). The staging dir itself stays
    // owner-only writable, so no one else can swap the file after verification.
    std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o644))
        .with_context(|| format!("setting permissions on {}", dest.display()))?;
    Ok(dest)
}

/// A staging directory apt's sandboxed `_apt` user can read — i.e. outside
/// `$HOME` (whose `0700` perms block it), under world-writable `/var/tmp` but as
/// an owner-only-writable subdirectory so no other unprivileged user can swap
/// the package between checksum verification and install.
fn staging_dir() -> Result<PathBuf> {
    let user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());
    let dir = PathBuf::from("/var/tmp").join(format!("josephine-{user}"));

    match std::fs::create_dir(&dir) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // Reuse only if it is genuinely ours — guards against a pre-planted
            // directory or symlink sitting in the shared /var/tmp.
            let meta = std::fs::symlink_metadata(&dir)
                .with_context(|| format!("inspecting {}", dir.display()))?;
            let ours = current_uid().is_some_and(|uid| meta.uid() == uid);
            if meta.file_type().is_symlink() || !meta.is_dir() || !ours {
                let d = dir.display();
                bail!(match i18n::lang() {
                    Lang::En => format!(
                        "the staging directory {d} already exists but isn't mine — stopping out of caution. Remove it and retry."
                    ),
                    Lang::Fr => format!(
                        "le dossier de préparation {d} existe déjà sans m'appartenir — par prudence, je m'arrête. Supprimez-le puis réessayez."
                    ),
                });
            }
        }
        Err(e) => return Err(e).with_context(|| format!("creating {}", dir.display())),
    }

    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755))
        .with_context(|| format!("setting permissions on {}", dir.display()))?;
    Ok(dir)
}

/// The caller's real UID, read from `/proc/self/status` (Linux-only).
fn current_uid() -> Option<u32> {
    parse_uid(&std::fs::read_to_string("/proc/self/status").ok()?)
}

fn parse_uid(status: &str) -> Option<u32> {
    status
        .lines()
        .find_map(|line| line.strip_prefix("Uid:"))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|uid| uid.parse().ok())
}

/// Verify the package against its published `.sha256`, if one exists. A missing
/// or unreachable checksum isn't fatal — we fall back to HTTPS trust.
fn verify(release: &ReleaseInfo, asset: &Asset, package: &Path) -> Integrity {
    let Some(sum_asset) = release.checksum_for(asset) else {
        return Integrity::Unverified;
    };
    let expected = ureq::get(&sum_asset.download_url)
        .header("User-Agent", USER_AGENT)
        .call()
        .ok()
        .and_then(|mut r| r.body_mut().read_to_string().ok())
        .and_then(|text| update::parse_sha256_line(&text));
    let actual = update::sha256_hex(package).ok();

    match (expected, actual) {
        (Some(e), Some(a)) if e == a => Integrity::Verified,
        (Some(_), Some(_)) => Integrity::Mismatch,
        _ => Integrity::Unverified,
    }
}

fn run_install(command: &[String], sudo: bool) -> Result<ExitStatus> {
    let mut argv: Vec<&str> = Vec::with_capacity(command.len() + 1);
    if sudo {
        argv.push("sudo");
    }
    argv.extend(command.iter().map(String::as_str));

    let (program, args) = argv.split_first().expect("non-empty install command");
    // Inherit stdio so the user can type their sudo password.
    Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("launching `{}`", argv.join(" ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_uid_from_status() {
        let sample = "Name:\tjosephine\nUid:\t1000\t1000\t1000\t1000\nGid:\t1000\t1000\n";
        assert_eq!(parse_uid(sample), Some(1000));
    }

    #[test]
    fn uid_absent_is_none() {
        assert_eq!(parse_uid("Name:\tx\nGid:\t0\n"), None);
    }
}
