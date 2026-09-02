# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **`status` now composes with scripts and status bars.** `josephine status
  --oneline` prints a single glanceable line (the worst glyph, then the most
  pressing check and its value), and `status` sets its exit code from the worst
  check it finds: `0` all clear, `1` something to look at, `2` something
  critical. That makes her a natural fit for Waybar / polybar / tmux / a shell
  prompt, without becoming a TUI — it stays a one-shot render. Codes `0`–`2`
  mean *health only*: when Joséphine can't answer at all she exits outside that
  band, following sysexits(3) — `64` for a malformed command line, `70` for a
  command that ran and failed — so a script can always tell "the machine is
  critical" from "Joséphine broke".
- **A gentle weekly digest.** `josephine report --since <window>` (e.g. `7d`,
  `24h`) appends a summary of the events over that window to the report — how
  many, what changed, and when. An opt-in systemd *user* timer
  (`josephine-report.timer`, shipped in the `.deb`/`.rpm`/AUR packages) runs it
  weekly and prints to the journal (`journalctl --user -u josephine-report`): a
  look back, never a push you didn't ask for.

### Removed

- **The example banner is gone.** `resources/banner.txt` was an ASCII angel from
  before Joséphine had a face, and the README offered it as a ready-to-use
  example — so anyone who followed it installed the old identity above their
  status screen. The feature itself is untouched: drop any ASCII or Braille art
  in `~/.config/josephine/banner.txt` and it still appears above the title,
  tinted with a gradient.

## [0.12.0] - 2026-07-27

### Added

- **Nix flake, with NixOS and Home Manager modules.** The repository is now a
  flake: `nix run github:systm-d/josephine` to try Joséphine, `nix profile
  install github:systm-d/josephine` to keep her, or add the flake as an input
  and set `services.josephine.enable = true` (on NixOS or Home Manager) to run
  the watcher as a systemd user service. The recipes live under `packaging/nix/`
  and the derivation builds from the committed `Cargo.lock`, so a new release
  needs nothing more than a tag. `josephine update` now recognises a
  `/nix/store` install and points you back to your configuration instead of
  trying to self-install into the read-only store.

### Changed

- **Joséphine has a face, and the words to match.** She is a *guardian
  spirit* now, not a guardian angel: a small fox-spirit who watches one
  machine. Four illustrations replace the hand-drawn SVG on the site — a
  portrait beside the hero, a scene where she reads your dials, one above
  the fourteen checks, and one of her asleep on the tower when you reach the
  bottom of the page. The wording follows everywhere it appeared, including
  the `--help` line and the crates.io description.
- **The callout no longer promises what she doesn't do.** It claimed she
  "sorts it out" with something close to a finger-snap; removing `josephine
  fix` in v0.11.0 disproved the first and orphaned the second. She notices,
  she tells you plainly, and she shows you the command rather than running
  it — which is now written into the project's own tone rule.

### Fixed

- **The `filesystem` check no longer false-alarms on NixOS.** NixOS keeps
  `/nix/store` read-only (via `boot.readOnlyNixStore`), so it shows up in
  `/proc/mounts` as a read-only bind mount of a normally-writable filesystem
  (`ext4`, `btrfs`, …) — exactly the shape the check reads as a disk silently
  going read-only. Mount points that are read-only by design are now skipped
  through a configurable `checks.filesystem.ignore_mounts` allowlist, which
  defaults to `/nix/store` and `/nix/.ro-store`; extend it for other immutable
  systems (for example `/usr` on an ostree-based distro).

## [0.11.0] - 2026-07-25

### Added

- **`doctor` now closes on what's left to do.** After the check-by-check exam,
  a grouped section lists every action worth taking — the exact unit to restart,
  the biggest consumer to stop — most severe first, and only when something
  needs doing. The remedies for all fourteen checks now live in one place
  (`josephine-core/src/remedy.rs`), shared with `josephine explain`, so the two
  commands cannot drift apart.

### Removed

- **`josephine fix` is gone.** It promised a repair it explicitly refused to
  perform — it only ever printed commands for you to run — and covered two of
  the fourteen checks. Its remedies moved into `doctor`, which covers all
  fourteen. Its closing promise did not change and did not move: Joséphine
  shows the way, you keep the wheel.

### Changed

- All fourteen remedies rewritten: seven so none sends you back to
  `josephine doctor` — inside doctor that was circular — and seven more
  tightened to fit the closing section's 80-column line. Every
  `josephine explain <check>` now prints a different remedy than before
  this release.

## [0.10.0] - 2026-07-24

### Added

- **Joséphine has more of a voice.** A small variety engine
  (`josephine-core/src/voice.rs`) means she no longer says things the exact
  same way every time: greetings, the "all clear" line, the `fix` sign-offs,
  the `daemon` status lines, the `notify test` message and the recovery
  notifications now rotate through several English **and** French phrasings.
  The *facts* of an alert — the number, the command to run — stay stable and
  precise; only the personality lines vary. A quiet, affectionate nod to
  *Joséphine, ange gardien*.
- **`doctor` now diagnoses.** It opens with a plain verdict — a clean bill of
  health, a note or two, or something that needs you now — before the
  check-by-check exam, living up to its name rather than merely inspecting.
- **The finger-snap.** `josephine fix` leans into Joséphine's signature
  gesture, in both its help text and its closing line (`✧`).

### Changed

- **`history` reads warmer.** A 24-hour tagline, guardian-style state
  transitions (`▲ attention → ● resolved` rather than `WARNING → RECOVERED`)
  and a closing line, in both languages.
- **Homebrew install is now a tap**, aligned with the `systm-d/claudine`
  packaging layout. The repository ships a discoverable `Formula/josephine.rb`
  rendered from the `packaging/homebrew/josephine.rb` template, so install with
  `brew tap systm-d/josephine https://github.com/systm-d/josephine` followed by
  `brew install josephine` (and it works from a `Brewfile`). This replaces
  `brew install …/josephine.rb`, which recent Homebrew rejects — installing a
  formula from a URL is no longer supported. Joséphine stays Linux-only
  (`depends_on :linux`): its checks and daemon rely on `/sys`, `/proc`,
  `systemctl` and `journalctl`, so no macOS/Windows builds are produced.
- **Release workflow restructured into parallel jobs** (`binaries`, `debian`,
  `fedora`, `release`, `homebrew`, `crates-io`), mirroring claudine. The
  `homebrew` job renders and commits `Formula/josephine.rb` on each tag, keeping
  the tap current. `.deb`/`.rpm`/`.tar.gz` artifacts keep their `.sha256`
  companions so `josephine update` can still verify downloads.

### Fixed

- Desktop-notification error context is now bilingual (it was French-only).
- Homebrew formula now builds the binary crate explicitly
  (`--path crates/josephine`); the previous default targeted the virtual
  workspace root and would fail to install.

## [0.9.0] - 2026-07-10

### Added

- **`filesystem` check** — flags writable filesystems unexpectedly remounted
  read-only (early sign of disk failure or corruption).
- **`timesync` check** — NTP clock synchronisation via `timedatectl`.
- **`security` check** — surfaces repeated failed login attempts in the last hour.
- **`josephine explain`** — bilingual what/why/remedy for all fourteen checks.

## [0.8.0] - 2026-07-09

### Added

- **`--json` output** for `status`, `doctor` and `report` — machine-readable
  JSON on stdout (no colour, no header) for scripting and monitoring. Severity
  is reported as `ok` / `warning` / `critical`.
- **Shell completions**: `josephine completions <bash|zsh|fish|…>` generates a
  completion script for your shell.
- **Terminal notifications**: the daemon now honours `notifications.terminal`,
  emitting alerts to its log/terminal channel (journal / foreground
  `daemon run`) alongside the desktop channel.
- **Localised `--help`**: `--help` / `--version` help text follows the
  configured `language` — English by default, French with `language: fr` — with
  no side effects on a fresh system.

## [0.7.1] - 2026-07-09

### Changed

- The whole CLI now reads as one system: the remaining commands (`report`,
  `clean`, `fix`, `update`, `daemon`, `config`, `notify`) drop the old
  `✨ Joséphine` banners and mascot copy for the direct "chaleur sobre" voice
  (English + French); `clean`, `fix` and `update` gain the sober `✦` header
  (`report` keeps its own body title, `daemon`/`config`/`notify` print no
  header); `status` and `doctor` share one footer; `history` events show
  human check labels. The `CURRENT_STATE` and `ROADMAP` docs are refreshed to
  the 0.7.0 baseline.

## [0.7.0] - 2026-07-09

### Changed

- **Sober CLI redesign ("Constellation sobre").** `status`, `doctor` and
  `history` drop the ASCII-angel banner, per-check emoji, hearts and the rounded
  advice box for a cleaner layout: a discreet `✦` header, status carried by
  coloured shape-glyphs (`●` ok · `▲` attention · `✕` critical, degrading to
  `[ok]/[!]/[x]` off a terminal), aligned columns and a one-line footer. The
  guardian-angel voice stays warm but becomes direct and concise, in both
  English and French — including the desktop notifications. A custom
  `banner.txt` is still honoured.

### Fixed

- The **inode** check no longer raises a false `critical` from snap mounts. Snaps
  are read-only `squashfs` images, always packed to 100 % inode usage by design;
  on most distros snapd mounts them under `/var/lib/snapd/snap/…`, which the old
  path-based filter (matching only `/snap`) missed, so a machine with many snaps
  would drown the real writable filesystems under dozens of 100 % lines. The
  check now reads `df -iPT` and skips read-only image filesystems by *type*
  (`squashfs` / `iso9660` / `erofs`), wherever they mount.

## [0.6.0] - 2026-07-03

### Added

- Each release now ships ready-to-use packaging recipes as assets: a Homebrew
  formula (`josephine.rb`) and an AUR `PKGBUILD`. The release workflow fills in
  the tag's version and the source tarball's real `sha256` in both, so on Linux
  you can
  `brew install https://github.com/systm-d/josephine/releases/latest/download/josephine.rb`
  (or `makepkg` from the attached `PKGBUILD`).

## [0.5.0] - 2026-07-02

### Added

- **Internationalisation.** Every user-facing string now ships in **English and
  French**: notifications, `status` / `doctor` / `history` output, all commands,
  check values, and config-validation errors. Set `language: fr` in the config
  for the French voice. The warm guardian-angel tone is preserved in both.

### Changed

- **English is now the default language** (Joséphine was French-only). French is
  one `language: fr` config line away. `CLAUDE.md`, README and docs updated.
- Landing site redesigned — warmer, livelier (halo hero, gradient feature cards,
  a commands reference, terminal chrome) and bilingual (EN default + FR), with
  real, anonymised example output.

## [0.4.1] - 2026-07-02

### Fixed

- `josephine daemon start` no longer refuses to start when the recorded PID has
  been recycled by an unrelated process. Daemon liveness now verifies the
  process is actually Joséphine (via `/proc/<pid>/cmdline`), so a stale pid file
  after a crash or logout no longer blocks a restart — which had silently
  starved `josephine history` of data. (#16)

## [0.4.0] - 2026-07-01

### Added

- Three new "guardian" checks, bringing the total to eleven:
  - **inode** — flags filesystems low on inodes (a disk can be "full" on inodes
    while still showing free space). Reads `df -iP`; runs as a normal user.
  - **smart** — per-disk SMART self-assessment via `smartctl`, an early warning
    of drive failure. Off by default (needs root); degrades to an informational
    "unavailable" rather than a false alarm.
  - **kernel** — counts recent kernel incidents (OOM kills, oops, BUG, panic)
    from `journalctl -k`, degrading gracefully when the journal isn't readable.
- Richer `josephine history`: per-metric **min / avg / max** and a 24-hour
  **sparkline** trend (`▁▂▃▅▇`) for CPU, memory, disk, temperature, network and
  battery — instead of only the daily maximum.

## [0.3.2] - 2026-07-01

### Changed

- `josephine update` stages the downloaded package under `/var/tmp` (in an
  owner-only-writable, ownership-checked subdirectory) instead of `$HOME`, so
  apt's sandboxed `_apt` user can read it — no more "Download is performed
  unsandboxed" warning during install. The package stays unreadable-for-write to
  other users, keeping the verify-then-install path safe on shared machines.

## [0.3.1] - 2026-07-01

### Changed

- `status` disk line is easier to read: `20% de « / » (192G / 937G)` — the
  percentage comes first (like the other checks) and the mount point is quoted
  so it can't be mistaken for a separator.

## [0.3.0] - 2026-07-01

### Added

- Two new checks, bringing the total to eight:
  - **network** — default-gateway reachability and latency. Stays strictly local
    (pings the LAN gateway, reads `/proc/net/route` and `/etc/resolv.conf`); no
    external host is ever contacted.
  - **battery** — charge level, charging state and health from
    `/sys/class/power_supply`; only warns while discharging and low, and reports
    "no battery" gracefully on desktops.
- `josephine report` — a dated, plain-text health snapshot, printed or written to
  a file with `--output`.
- `josephine clean` — previews reclaimable space (user cache, thumbnails, `/tmp`,
  systemd journals) by default; `--apply` clears the always-safe thumbnail cache
  and shows the exact commands for the privileged reclaims.
- `josephine fix` — guided remediation: surfaces failed services and disk
  pressure with the precise command to fix each. Advisory only; nothing
  privileged runs on its own.
- `josephine notify test` — sends a test desktop notification to verify libnotify.
- `josephine config edit` — opens the config in `$EDITOR`, then re-validates it.

### Changed

- Replaced the `clean`/`fix`/`report` and `config edit` stubs with working
  commands.
- Roadmap and current-state docs refreshed to the 0.3.0 baseline; dropped an
  obsolete cargo-deny advisory exception that no longer matched any crate.

## [0.2.2] - 2026-07-01

### Added

- `josephine update`: checks GitHub Releases and, on request, downloads and
  installs the package matching your install channel (`.deb`/`.rpm`), leaving the
  privileged step (`sudo`) to you. The network is touched only on this explicit
  command — never in the background. `--check` reports without installing;
  `--yes` skips the confirmation prompt. Release artifacts now ship a `.sha256`
  per package so the download can be integrity-checked.

## [0.2.1] - 2026-07-01

### Changed

- `status` header is now a sober title block; removed the built-in ASCII-art
  angel avatar. A custom `<config>/banner.txt` is still honoured when present.

## [0.2.0] - 2026-07-01

### Added

- New `updates` check: counts available package updates via apt / dnf / pacman.
- Redesigned `status` screen: angel banner, per-check emoji icons, a system load
  line (from `/proc/loadavg`), colour-coded values and a rounded advice box.
- Customizable banner: `status` uses `<config>/banner.txt` (any ASCII/Braille
  art, tinted with a gradient) when present, else the built-in angel.
- `josephine doctor --verbose`: adds numeric thresholds, the top 10 processes
  (CPU & memory) and each check's collection interval.
- Project landing site (Zola) deployed to GitHub Pages.
- `josephine daemon run` foreground subcommand for systemd supervision.
- Packaging: systemd user unit, deb/rpm metadata, AUR PKGBUILD, Homebrew (Linux)
  formula, and a tag-driven release workflow (GitHub Releases + crates.io).
- Committed `Cargo.lock` for reproducible builds.
- Continuous integration: lint, multi-distro test matrix (Ubuntu 22.04/24.04,
  Fedora 40/41), coverage (informational), supply-chain security checks, and a
  criterion benchmark.

### Changed

- Migrated the workspace to Rust edition 2024 (MSRV 1.85).
- Renamed the binary crate `josephine-cli` to `josephine`.
- Dual-licensed under MIT OR Apache-2.0.
- Adopted the shared `rust-cli-template` conventions (toolchain, rustfmt, lints,
  release profile, integration tests).
- Reworked the CLI copy (stubs, `config`, daemon lifecycle, status/doctor/history
  footers) for a warmer, wittier guardian-angel voice.
- `josephine doctor` now renders a single unified table (one row per check)
  instead of one box per check.

### Fixed

- `josephine config edit` now prints a friendly "coming soon" message and exits 0,
  instead of surfacing as an error (exit 1) like the other stub commands.

## [0.1.0] - 2026-06-30

### Added

- Initial release: `status`, `doctor`, `history`, `daemon`, `config` commands.
- Five checks: cpu, memory, disk, temperature, systemd.
- Background daemon with desktop notifications and a 90-day SQLite history.
