//! Self-update helpers: decide whether a newer Joséphine exists on GitHub
//! Releases, which package fits the current install, and how to install it.
//!
//! This module is deliberately network-free so it stays unit-testable: the HTTP
//! calls live in the CLI (`update_cmd`), which feeds the JSON here via
//! [`parse_release`]. Joséphine only reaches the network on an explicit
//! `josephine update` — never in the background (the "100 % local" rule).

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use serde::Deserialize;

/// `owner/repo` published on GitHub Releases.
pub const REPO: &str = "systm-d/josephine";

/// GitHub REST endpoint for the latest published release.
pub const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/systm-d/josephine/releases/latest";

/// The version this binary was built from.
pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// A published release, distilled from the GitHub API payload.
#[derive(Debug, Clone)]
pub struct ReleaseInfo {
    pub tag: String,
    pub version: String,
    pub html_url: String,
    pub notes: String,
    pub assets: Vec<Asset>,
}

/// A downloadable file attached to a release.
#[derive(Debug, Clone)]
pub struct Asset {
    pub name: String,
    pub download_url: String,
    pub size: u64,
}

impl ReleaseInfo {
    /// The release asset matching a channel's package suffix, if any.
    pub fn asset_for(&self, channel: InstallChannel) -> Option<&Asset> {
        let suffix = channel.package_suffix()?;
        self.assets.iter().find(|a| a.name.ends_with(suffix))
    }

    /// The published `.sha256` companion for an asset, if one was uploaded.
    pub fn checksum_for(&self, asset: &Asset) -> Option<&Asset> {
        let name = format!("{}.sha256", asset.name);
        self.assets.iter().find(|a| a.name == name)
    }
}

// --- GitHub JSON mirrors -----------------------------------------------------

#[derive(Deserialize)]
struct ApiRelease {
    tag_name: String,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    assets: Vec<ApiAsset>,
}

#[derive(Deserialize)]
struct ApiAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

/// Parse a GitHub "latest release" JSON payload into a [`ReleaseInfo`].
pub fn parse_release(json: &str) -> Result<ReleaseInfo> {
    let api: ApiRelease =
        serde_json::from_str(json).context("réponse de GitHub illisible (JSON inattendu)")?;
    let version = api.tag_name.trim_start_matches('v').to_string();
    Ok(ReleaseInfo {
        tag: api.tag_name,
        version,
        html_url: api.html_url,
        notes: api.body,
        assets: api
            .assets
            .into_iter()
            .map(|a| Asset {
                name: a.name,
                download_url: a.browser_download_url,
                size: a.size,
            })
            .collect(),
    })
}

// --- Version comparison ------------------------------------------------------

/// Where the local build sits relative to the latest published release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateStatus {
    /// Running the newest published version.
    UpToDate,
    /// A newer version is available (carries that version string).
    Available(String),
    /// Local build is newer than anything published (a dev / pre-release build).
    Ahead,
}

/// Compare two version strings using semantic-versioning rules.
///
/// Falls back to an exact string comparison if either side isn't valid semver,
/// erring toward "up to date" only on an exact match.
pub fn compare(current: &str, latest: &str) -> UpdateStatus {
    use std::cmp::Ordering;
    match (
        semver::Version::parse(current),
        semver::Version::parse(latest),
    ) {
        (Ok(cur), Ok(new)) => match new.cmp(&cur) {
            Ordering::Greater => UpdateStatus::Available(latest.to_string()),
            Ordering::Equal => UpdateStatus::UpToDate,
            Ordering::Less => UpdateStatus::Ahead,
        },
        _ if current == latest => UpdateStatus::UpToDate,
        _ => UpdateStatus::Available(latest.to_string()),
    }
}

// --- Install channel detection ----------------------------------------------

/// How this binary was most likely installed — drives the update strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallChannel {
    Deb,
    Rpm,
    Pacman,
    Cargo,
    Homebrew,
    Nix,
    Tarball,
    Unknown,
}

impl InstallChannel {
    /// The release-asset filename suffix to download for this channel, when the
    /// update can be applied by installing a package.
    fn package_suffix(self) -> Option<&'static str> {
        match self {
            InstallChannel::Deb => Some(".deb"),
            InstallChannel::Rpm => Some(".rpm"),
            InstallChannel::Tarball => Some(".tar.gz"),
            _ => None,
        }
    }
}

/// Best-effort detection of how the running binary was installed.
pub fn detect_channel() -> InstallChannel {
    let exe = std::env::current_exe().unwrap_or_default();
    detect_channel_for(&exe)
}

fn detect_channel_for(exe: &Path) -> InstallChannel {
    let path = exe.to_string_lossy();

    // Cargo and Homebrew are unambiguous straight from the path.
    if let Some(channel) = channel_from_path(&path) {
        return channel;
    }
    // A system path: ask each package database whether it owns this file.
    if package_owns("dpkg", &["-S"], exe) {
        return InstallChannel::Deb;
    }
    if package_owns("rpm", &["-qf"], exe) {
        return InstallChannel::Rpm;
    }
    if package_owns("pacman", &["-Qo"], exe) {
        return InstallChannel::Pacman;
    }
    // Sitting in a system location without a package owner → a manual tarball copy.
    if path.starts_with("/usr/") || path.starts_with("/opt/") {
        return InstallChannel::Tarball;
    }
    InstallChannel::Unknown
}

/// The path-only part of channel detection (no system calls) — unit-testable.
fn channel_from_path(path: &str) -> Option<InstallChannel> {
    if path.contains("/.cargo/") {
        Some(InstallChannel::Cargo)
    } else if path.contains("linuxbrew") || path.contains("/Cellar/") {
        Some(InstallChannel::Homebrew)
    } else if path.starts_with("/nix/store/") {
        Some(InstallChannel::Nix)
    } else {
        None
    }
}

fn package_owns(bin: &str, args: &[&str], exe: &Path) -> bool {
    Command::new(bin)
        .args(args)
        .arg(exe)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// --- Install plan ------------------------------------------------------------

/// What to do once we know the channel and (maybe) have a package on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallPlan {
    /// Run this argv to install; the caller prepends `sudo` when `sudo` is true.
    Run { command: Vec<String>, sudo: bool },
    /// We won't drive a foreign package manager — show the user the command.
    Manual(String),
}

/// Decide how to install `package` for the detected `channel`.
pub fn install_plan(channel: InstallChannel, package: &Path) -> InstallPlan {
    let pkg = package.display().to_string();
    match channel {
        InstallChannel::Deb => InstallPlan::Run {
            command: vec!["apt".into(), "install".into(), pkg],
            sudo: true,
        },
        InstallChannel::Rpm => InstallPlan::Run {
            command: vec!["dnf".into(), "install".into(), pkg],
            sudo: true,
        },
        InstallChannel::Pacman => InstallPlan::Manual(
            crate::i18n::t(
                "On Arch, updates go through the AUR: `yay -S josephine` (or your AUR helper of choice).",
                "Sur Arch, la mise à jour passe par l'AUR : `yay -S josephine` (ou l'assistant AUR de votre choix).",
            )
            .into(),
        ),
        InstallChannel::Homebrew => InstallPlan::Manual(
            crate::i18n::t(
                "With Homebrew: `brew upgrade josephine`.",
                "Via Homebrew : `brew upgrade josephine`.",
            )
            .into(),
        ),
        // A Nix install lives in the read-only /nix/store: nothing to install in
        // place. The update comes from the user's own configuration.
        InstallChannel::Nix => InstallPlan::Manual(
            crate::i18n::t(
                "On Nix, updates go through your configuration: bump the `josephine` flake input (or your channel) and rebuild.",
                "Sur Nix, la mise à jour passe par votre configuration : actualisez l'entrée de flake `josephine` (ou votre canal), puis reconstruisez.",
            )
            .into(),
        ),
        InstallChannel::Cargo => InstallPlan::Manual(match crate::i18n::lang() {
            crate::i18n::Lang::En => {
                format!("With cargo: `cargo install --git https://github.com/{REPO} josephine`.")
            }
            crate::i18n::Lang::Fr => {
                format!("Via cargo : `cargo install --git https://github.com/{REPO} josephine`.")
            }
        }),
        InstallChannel::Tarball | InstallChannel::Unknown => {
            InstallPlan::Manual(match crate::i18n::lang() {
                crate::i18n::Lang::En => format!(
                    "Grab the latest archive from https://github.com/{REPO}/releases/latest and replace your binary."
                ),
                crate::i18n::Lang::Fr => format!(
                    "Récupérez la dernière archive sur https://github.com/{REPO}/releases/latest et remplacez votre binaire."
                ),
            })
        }
    }
}

// --- Checksum ----------------------------------------------------------------

/// Compute the lowercase hex SHA-256 of a file.
pub fn sha256_hex(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut file =
        std::fs::File::open(path).with_context(|| format!("opening {}", path.display()))?;
    let mut hasher = Sha256::new();
    // sha2 0.11 dropped the `io::Write` impl on hashers, so feed it by hand
    // rather than through `io::copy`.
    let mut buf = [0u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buf)
            .context("reading the file to checksum")?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }

    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(hex, "{byte:02x}");
    }
    Ok(hex)
}

/// Extract the hex digest from a `sha256sum`-style line (`<hash>  <file>`).
pub fn parse_sha256_line(text: &str) -> Option<String> {
    text.split_whitespace().next().map(str::to_lowercase)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    const SAMPLE: &str = r#"{
        "tag_name": "v0.3.0",
        "html_url": "https://github.com/systm-d/josephine/releases/tag/v0.3.0",
        "body": "Notes",
        "assets": [
            {"name": "josephine_0.3.0-1_amd64.deb", "browser_download_url": "https://example/deb", "size": 10},
            {"name": "josephine_0.3.0-1_amd64.deb.sha256", "browser_download_url": "https://example/deb.sha256", "size": 1},
            {"name": "josephine-0.3.0-1.x86_64.rpm", "browser_download_url": "https://example/rpm", "size": 20},
            {"name": "josephine-0.3.0-x86_64-unknown-linux-gnu.tar.gz", "browser_download_url": "https://example/tgz", "size": 30}
        ]
    }"#;

    #[test]
    fn parse_release_extracts_version_and_assets() {
        let r = parse_release(SAMPLE).unwrap();
        assert_eq!(r.tag, "v0.3.0");
        assert_eq!(r.version, "0.3.0");
        assert_eq!(r.assets.len(), 4);
    }

    #[test]
    fn compare_detects_all_orderings() {
        assert_eq!(compare("0.2.1", "0.2.1"), UpdateStatus::UpToDate);
        assert_eq!(
            compare("0.2.1", "0.3.0"),
            UpdateStatus::Available("0.3.0".into())
        );
        // Numeric, not lexical: 0.2.10 must beat 0.2.9.
        assert_eq!(
            compare("0.2.9", "0.2.10"),
            UpdateStatus::Available("0.2.10".into())
        );
        assert_eq!(compare("0.3.0", "0.2.1"), UpdateStatus::Ahead);
    }

    #[test]
    fn asset_for_matches_by_suffix() {
        let r = parse_release(SAMPLE).unwrap();
        assert_eq!(
            r.asset_for(InstallChannel::Deb).unwrap().name,
            "josephine_0.3.0-1_amd64.deb"
        );
        assert_eq!(
            r.asset_for(InstallChannel::Rpm).unwrap().name,
            "josephine-0.3.0-1.x86_64.rpm"
        );
        assert_eq!(
            r.asset_for(InstallChannel::Tarball).unwrap().name,
            "josephine-0.3.0-x86_64-unknown-linux-gnu.tar.gz"
        );
        assert!(r.asset_for(InstallChannel::Cargo).is_none());
    }

    #[test]
    fn deb_asset_is_not_the_sha256_sidecar() {
        let r = parse_release(SAMPLE).unwrap();
        let deb = r.asset_for(InstallChannel::Deb).unwrap();
        assert!(!deb.name.ends_with(".sha256"));
        assert_eq!(
            r.checksum_for(deb).unwrap().name,
            "josephine_0.3.0-1_amd64.deb.sha256"
        );
    }

    #[test]
    fn channel_from_path_spots_cargo_and_brew() {
        assert_eq!(
            channel_from_path("/home/x/.cargo/bin/josephine"),
            Some(InstallChannel::Cargo)
        );
        assert_eq!(
            channel_from_path("/home/linuxbrew/.linuxbrew/bin/josephine"),
            Some(InstallChannel::Homebrew)
        );
        assert_eq!(
            channel_from_path("/nix/store/abcd1234-josephine-0.11.0/bin/josephine"),
            Some(InstallChannel::Nix)
        );
        assert_eq!(channel_from_path("/usr/bin/josephine"), None);
    }

    #[test]
    fn install_plan_uses_apt_for_deb() {
        let plan = install_plan(InstallChannel::Deb, Path::new("/tmp/j.deb"));
        assert_eq!(
            plan,
            InstallPlan::Run {
                command: vec!["apt".into(), "install".into(), "/tmp/j.deb".into()],
                sudo: true,
            }
        );
    }

    #[test]
    fn install_plan_defers_to_third_party_managers() {
        assert!(matches!(
            install_plan(InstallChannel::Homebrew, Path::new("")),
            InstallPlan::Manual(_)
        ));
        assert!(matches!(
            install_plan(InstallChannel::Pacman, Path::new("")),
            InstallPlan::Manual(_)
        ));
        // A Nix install is immutable, so it can only ever be a manual pointer
        // back to the user's configuration.
        assert!(matches!(
            install_plan(InstallChannel::Nix, Path::new("")),
            InstallPlan::Manual(_)
        ));
    }

    #[test]
    fn sha256_line_takes_first_field() {
        assert_eq!(
            parse_sha256_line("abc123  josephine.deb\n").as_deref(),
            Some("abc123")
        );
    }

    /// Write `bytes` to a uniquely named file under the temp dir and return it.
    fn temp_file_with(tag: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "josephine-sha256-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::write(&path, bytes).expect("writing the temp file");
        path
    }

    #[test]
    fn sha256_hex_matches_the_known_vector() {
        let path = temp_file_with("known", b"hello");
        let digest = sha256_hex(&path).expect("hashing");
        std::fs::remove_file(&path).ok();

        // `printf 'hello' | sha256sum`
        assert_eq!(
            digest,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    /// The file is read through a 64 KiB buffer, so anything larger exercises
    /// the loop's chunk boundaries — where an off-by-one would hide.
    #[test]
    fn sha256_hex_spans_several_buffer_fills() {
        use sha2::{Digest, Sha256};

        let bytes: Vec<u8> = (0..200_000u32).map(|i| (i % 251) as u8).collect();
        let path = temp_file_with("large", &bytes);
        let from_file = sha256_hex(&path).expect("hashing");
        std::fs::remove_file(&path).ok();

        // Same bytes hashed in one shot: the chunked read must agree.
        let one_shot = Sha256::digest(&bytes);
        let expected: String = one_shot.iter().map(|b| format!("{b:02x}")).collect();

        assert_eq!(from_file, expected);
    }

    #[test]
    fn sha256_hex_of_an_empty_file_is_the_empty_digest() {
        let path = temp_file_with("empty", b"");
        let digest = sha256_hex(&path).expect("hashing");
        std::fs::remove_file(&path).ok();

        assert_eq!(
            digest,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
