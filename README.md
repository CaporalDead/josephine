<p align="center">
  <img src="resources/social-preview-en.png" alt="Joséphine — your computer's quiet guardian spirit" width="720">
</p>

<h1 align="center">Joséphine</h1>

<p align="center"><em>Your computer's quiet guardian spirit.</em></p>

<p align="center">
  <a href="https://github.com/systm-d/josephine/actions/workflows/ci.yml"><img src="https://github.com/systm-d/josephine/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="https://github.com/systm-d/josephine/releases/latest"><img src="https://img.shields.io/github/v/release/systm-d/josephine?color=e0a458&label=release" alt="Latest release"></a>
  <img src="https://img.shields.io/badge/platform-Linux-333" alt="Linux only">
  <a href="#license"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue" alt="License: MIT OR Apache-2.0"></a>
</p>

Joséphine watches your machine silently and only speaks up when it actually
helps. She keeps an eye on CPU, memory, disk, temperature, systemd services and
pending updates, detects trouble early, and sends warm, plain-language desktop
notifications — never intrusive, always local. **No data ever leaves your
computer.**

> Joséphine speaks **English by default**; set `language: fr` in
> `~/.config/josephine/config.yaml` for her French voice. The same warm,
> protective voice in both.

## Features

- **Fourteen built-in checks** — CPU, memory, disk, temperature, systemd services,
  package updates (apt / dnf / pacman), local network (gateway latency), battery,
  inode usage, SMART disk health (opt-in), kernel incidents (OOM / oops),
  read-only filesystem remounts, NTP clock sync and recent failed logins.
- **Warm notifications** — plain-language desktop alerts that escalate only when
  it helps; never `ERROR` / `FATAL` / `PANIC`. She varies her phrasing so it
  never feels canned — while the facts (the number, the command) stay exact.
- **Background daemon** — a lightweight systemd *user* service that watches
  continuously and records a rolling **24-hour history** (local SQLite).
- **At-a-glance `status`** — colour-coded summary with a **customizable banner**.
- **Detailed `doctor`** — opens with a plain verdict (clean bill of health, a
  note or two, or something that needs you now), then check-by-check
  diagnostics; `--verbose` adds thresholds, the top 10 processes and each
  check's collection interval.
- **`explain`** — what each check watches, why it matters, and how to act.
- **Self-update** — `josephine update` checks GitHub Releases and installs the
  package matching your install (`.deb` / `.rpm`); reaches the network only when
  you ask.
- **Bilingual** — English by default, French with `language: fr` in the config;
  warm, never alarmist, in both.
- **100% local** — no cloud, no telemetry, Linux-native (systemd, `/sys`,
  libnotify).

## Installation

Every `v*.*.*` tag triggers the [Release](.github/workflows/release.yml)
workflow, which publishes artifacts for the common Linux targets. Joséphine is
Linux-native (systemd, `/sys`, `/proc`, libnotify) — there are **no macOS or
Windows builds**.

| Platform        | Artifact                          |
| --------------- | --------------------------------- |
| Linux (generic) | `josephine-linux-x86_64.tar.gz`   |
| Debian / Ubuntu | `josephine_<version>_amd64.deb`   |
| Fedora / RHEL   | `josephine-<version>.x86_64.rpm`  |
| Arch Linux      | AUR — `packaging/aur/PKGBUILD`    |
| NixOS / Nix     | Flake: `nix run github:systm-d/josephine` |

```sh
# Debian / Ubuntu
sudo dpkg -i josephine_*_amd64.deb
# Fedora / RHEL
sudo rpm -i josephine-*.x86_64.rpm
```

### Package managers

**Homebrew** — this repository is a tap (builds from source, requires Rust;
Linux-only, `depends_on :linux`):

```sh
brew tap systm-d/josephine https://github.com/systm-d/josephine
brew install josephine
```

Or pin it in a `Brewfile`:

```ruby
tap "systm-d/josephine", "https://github.com/systm-d/josephine"
brew "josephine"
```

**Arch Linux — AUR:** each release ships a ready-to-use `PKGBUILD`
([`packaging/aur/PKGBUILD`](packaging/aur/PKGBUILD)):

```sh
curl -LO https://github.com/systm-d/josephine/releases/latest/download/PKGBUILD
makepkg -si
```

**Nix / NixOS:** this repository is a flake. Try Joséphine without installing
anything:

```sh
nix run github:systm-d/josephine -- status
```

Install it imperatively into your profile:

```sh
nix profile install github:systm-d/josephine
```

Or wire it into your system by adding the flake as an input:

```nix
# flake.nix
inputs.josephine.url = "github:systm-d/josephine";
```

On NixOS, import the module and enable the background watcher (it runs as a
systemd *user* service, like the other packages):

```nix
imports = [ inputs.josephine.nixosModules.default ];
services.josephine.enable = true;
```

With Home Manager, the module has the same shape:

```nix
imports = [ inputs.josephine.homeManagerModules.default ];
services.josephine.enable = true;
```

The package is also exposed as `packages.<system>.default` and through
`overlays.default`, if you would rather add it to `environment.systemPackages`
yourself. Because a Nix install lives in the read-only store, `josephine update`
will not self-install; it points you back to your configuration instead.

### From source

Requires Rust 1.85+.

```sh
cargo install --git https://github.com/systm-d/josephine josephine
```

## Usage

```sh
josephine               # quick status summary (default)
josephine status        # CPU, memory, disk, temperature, systemd, updates at a glance
josephine status --oneline  # one compact line for a status bar (Waybar, polybar, tmux)
josephine doctor        # full diagnostics, and what's left to do
josephine doctor -v     # verbose: thresholds, top 10 processes, intervals
josephine history       # last 24 h: min/avg/max + sparkline trends, and events
josephine daemon start  # run the background watcher
josephine daemon status # daemon state (PID, uptime)
josephine config show   # print the current configuration
josephine config edit   # edit the config in $EDITOR, then re-validate
josephine report        # dated plain-text health report (-o writes to a file)
josephine clean         # preview reclaimable disk space (--apply clears caches)
josephine explain       # what each check watches and how to act
josephine explain disk  # full explanation for one check
josephine notify test   # send a test desktop notification
josephine update        # check GitHub for a newer version and install it
josephine --version
```

`josephine status` sets its exit code from the worst check it finds — `0` all
clear, `1` something to look at, `2` something critical — so it drops straight
into a script or a status bar. Pair it with `--oneline` for a single glanceable
line, or `--json` for the full machine-readable view.

Those three codes only ever mean health. If Joséphine can't answer at all she
exits outside that band, following `sysexits(3)`: `64` if the command line was
malformed, `70` if the command ran and failed. So a status bar can tell a
critical machine from a broken invocation.

`josephine update` reaches the network only when you run it — never in the
background. It detects how Joséphine was installed (`.deb`/`.rpm`/…), downloads
the matching package, verifies its checksum, and hands the privileged install
step (`sudo`) to you.

To keep Joséphine watching across reboots, enable the bundled systemd **user**
unit ([`packaging/systemd/josephine.service`](packaging/systemd/josephine.service)):

```sh
systemctl --user enable --now josephine
```

Configuration lives at `~/.config/josephine/config.yaml` (created on first run).
History and the daemon's state live under `~/.local/share/josephine/`.

The `status` header is deliberately sober. Want a flourish? Drop any ASCII/Braille
art in `~/.config/josephine/banner.txt` and it appears above the title, tinted
with a gradient.

## Documentation

- [Architecture](docs/ARCHITECTURE.md) · [Development](docs/DEVELOPMENT.md) · [Current state](docs/CURRENT_STATE.md) · [Roadmap](docs/ROADMAP.md)
- [Contributing](CONTRIBUTING.md) · [Conventions](CONVENTIONS.md) · [Code of Conduct](CODE_OF_CONDUCT.md) · [Security](SECURITY.md)
- Website: <https://systm-d.github.io/josephine/>
- En français : [README.fr.md](README.fr.md)

## Development

```sh
cargo build
cargo test --workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the workflow and quality gate.

New contributors are welcome, and there is usually something small to pick up:

- Good first issues: <https://github.com/systm-d/josephine/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22>
- Where help is wanted: <https://github.com/systm-d/josephine/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22>

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option.
