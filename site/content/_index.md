+++
title = "Joséphine"

[extra]
eyebrow = "Local Linux guardian"
tagline = "Your machine, watched over — quietly."
lede = "You already have htop, smartctl and journalctl. Joséphine is the one keeping watch between glances — she notices the slow problems (a filling disk, a fading SSD, a service that quietly died) and speaks up before they become your problem."
cta = "View on GitHub"
cta2 = "Install"
readout_alt = "Example josephine status output: sixteen checks, one flagged for attention"
+++

<section id="install" class="reveal">
<p class="eyebrow">Install</p>

## Up in a minute

Grab a package from the [latest release](https://github.com/systm-d/josephine/releases/latest):

```sh
# Debian / Ubuntu
sudo dpkg -i josephine_*_amd64.deb

# Fedora / RHEL
sudo rpm -i josephine-*.x86_64.rpm
```

Prefer to build it yourself? `cargo install --git https://github.com/systm-d/josephine josephine` (Rust 1.85+). On Linux with Homebrew, `brew tap systm-d/josephine https://github.com/systm-d/josephine && brew install josephine` builds from source.

On NixOS or with Nix, Joséphine is a flake: `nix run github:systm-d/josephine -- status` to try her, or add her module and set `services.josephine.enable = true` (on NixOS or Home Manager) to run the daemon as a systemd **user** service.

To keep Joséphine watching across reboots, enable the bundled systemd **user** unit:

```sh
systemctl --user enable --now josephine
```

<p class="callout"><strong>Joséphine is a guardian angel, not a dashboard.</strong> She notices when your machine needs a hand, tells you plainly what it needs — and shows you the command rather than running it behind your back. For people who'd rather not have to keep an eye on it themselves.</p>

> Joséphine speaks English by default — set `language: fr` in the config for her French voice. The warm, direct tone is preserved in both.
</section>
