# Joséphine - Foresight & guardian depth (six increments)

**Date:** 2026-07-28
**Status:** Draft - foresight framing pending owner input (see issue #57)
**Scope:** two CLI ergonomics, three checks, one foresight engine, one digest
**Target releases:** 0.13.0 -> 0.15.0 (one increment per PR)
**Branch:** per-increment (`feat/status-oneline`, `feat/check-reboot`, ...)

---

## Context

The "notice / diagnose" spine is complete: fourteen checks, a daemon, and a
`doctor` that now closes on what's left to do. What is missing is the pillar the
roadmap named but barely started - **Prévoyance**, anticipating rather than only
observing.

The site's own promise is that Joséphine *"notices the slow problems (a filling
disk, a fading SSD, a service that quietly died) and speaks up before they
become your problem."* Yet the SQLite history she already keeps
(`storage.rs:74`, table `metrics`) is only ever **displayed** - the `history`
sparklines - and never **projected**. The data to keep that promise is on disk;
nothing reads it forward.

This program closes that gap and deepens the "slow problem" coverage, without
breaking a single locked rule:

- 100% local, no cloud, no telemetry, no accounts.
- Linux-only (systemd, /sys, /proc, libnotify).
- Invisible by default; warm, never alarmist.
- Deterministic observation only.
- She shows the command; she does not run privileged actions herself.

Each increment is weighed against the roadmap's add test - *does this help
Joséphine better protect the machine?* - and ships as **one focused PR**.

### Decisions locked during brainstorming

- **Shape:** an umbrella program of six small, independent increments, sequenced
  from safe-and-quick to the flagship, mirroring the CLI increments A->D.
- **Foresight is deterministic:** a plain least-squares line fit over the
  existing history - reproducible arithmetic, no learned model. Whether that
  sits inside the "no AI for monitoring" rule is the one product call reserved
  for the owner (issue #57). The rest of this document assumes "yes"; if the
  answer is "no", increment E is dropped and A/B/C/D/F stand on their own.
- **Foresight MVP is quiet:** `doctor`-only, **no notification**, **no JSON
  change** - the same restraint the remedies increment chose
  (`2026-07-25-josephine-remedies-in-doctor-design.md:202`). A forecast
  notification is a deliberate non-goal for now.
- **Nothing here lands before the owner has had a say**, especially foresight.

### Independencies

B, C, D, E and F are independent at the code level; only **E benefits from D**
(it can also project SSD wear). A is pure ergonomics and unblocks glanceable
use. So A/B/C/D/F can proceed even if E is declined.

---

## Increment A - exit codes + `status --oneline`

**Goal.** Let Joséphine be *glanced at* from a status bar, and let scripts read
her verdict.

Today `cli::run` returns only `ExitCode::SUCCESS` or `ExitCode::from(1)` on a
command error (`cli.rs:97-105`); the health verdict never reaches the exit code.

- `josephine status --oneline` prints one compact line: the worst glyph plus the
  worst check, e.g. `▲ disk 88%` (degrading to `[!] disk 88%` off a TTY, using
  the shape+colour glyphs already in `output/style.rs`).
- `josephine status` maps the worst severity across checks to an exit code:
  **0** ok, **1** attention, **2** critical. `dispatch` threads a `Severity`
  out of `status_cmd::run` -> `run` -> `ExitCode` (`cli.rs:97-105, 166-186`).

**Files.** `cli.rs`, `commands/status_cmd.rs`, `output/status.rs`. JSON output
(`output/json.rs`) is unchanged.

**Why it passes the test.** "Watched between glances" is literally her tagline;
this makes her embeddable in Waybar / polybar / i3blocks / tmux / a shell
prompt. It is **not** a TUI (the rejected `watch`): no full-screen, no live
loop, just a one-shot formatter. It stays opt-in, so "invisible by default"
holds.

**Tests.** `--oneline` rendering (TTY and plain); the severity -> exit-code
mapping for each band.

**Scope note for the owner.** Exit codes are a small behaviour change for anyone
already scripting `josephine status`; today it exits 0 on a critical machine. I
read that as a fix, but it is worth a line in the changelog.

---

## Increment B - `reboot` check

**Goal.** Say when a reboot is required for an update to take effect - the quiet
gap after `updates` reports "all applied".

Detection, best-effort and degrading to "unavailable" when undeterminable:

1. `/run/reboot-required` (or `/var/run/reboot-required`) - Debian/Ubuntu.
2. `needs-restarting -r` exit code - Fedora/RHEL (`dnf-utils`).
3. else compare the running kernel (`uname -r`) against the newest installed
   kernel (`/lib/modules`, `/boot`).

Metric `reboot_required` (0/1), `warning = 1`, no critical (a pending reboot is
attention, not an emergency).

**Files.** The full seven-step add (`ARCHITECTURE.md`): `config.rs`
(`RebootCheckConfig` + defaults + `validate_reboot`), `checks/reboot.rs`,
`checks/mod.rs` (`build_checks`, `interval_for_check`), `scheduler.rs`
(`thresholds_for`), `messages.rs` (alert + recovery), `remedy.rs` (Advice),
`output/style.rs` (`check_label`, `primary_metric`). Bumps the
`all_fourteen_checks_have_advice` guard in `remedy.rs` to fifteen.

**Tests.** Parsing/decision from each source; the "unavailable" degrade.

---

## Increment C - `pressure` check (PSI)

**Goal.** Catch the machine *struggling* before it fails - memory thrashing
before the OOM killer, IO saturation before everything stalls.

Read `/proc/pressure/{cpu,memory,io}` and take the `some avg60` figure (percent
of the last 60 s during which at least one task was stalled). Primary metric
`memory_pressure_avg60` (suggested `warning = 20`, `critical = 40`); `cpu` and
`io` pressure recorded as secondary metrics for `history`/`doctor`. Degrades to
"unavailable" when PSI is not present (older kernels, or `CONFIG_PSI` off).

**Why it passes the test.** PSI is the modern, deterministic signal of real
pressure - finer than a raw usage percentage, which can read 95% and be
perfectly happy. Memory PSI rising is the earliest honest warning of a swap
death-spiral.

**Files.** Full seven-step add; bumps the advice guard to sixteen.

**Tests.** Parsing the `some`/`full` lines; the missing-PSI degrade.

---

## Increment D - SSD/NVMe wear (extends `smart`)

**Goal.** Turn the SMART check from a binary "passing / failing" into the
gradual signal that actually matters for an SSD: how much of its rated write
life is gone.

Beside the current `smartctl -H` (`smart.rs:144`), read wear:

- NVMe: `Percentage_Used` from `smartctl -j <dev>` (`nvme_smart_health_information_log.percentage_used`).
- SATA SSD: `Media_Wearout_Indicator` (233) / `SSD_Life_Left` (231) /
  `Wear_Leveling_Count` (177), normalised to "percent used".

New metric `smart_wear_percent` (worst disk; suggested `warning = 80`,
`critical = 90`). The check stays opt-in (root), and reports wear only where a
device exposes it - spinning disks and SMART-less drives simply don't
contribute the metric.

**Files.** `checks/smart.rs` (a `device_wear` fn, JSON parsing, the second
metric), `config.rs` (wear thresholds on `SmartCheckConfig`), `messages.rs`,
`remedy.rs`, `output/style.rs`, and parse-fixture tests for an NVMe and a SATA
sample.

**Why it passes the test.** It is the very "fading SSD" the site names, made
measurable years before `-H` flips to FAILED. And it feeds increment E.

---

## Increment E - Foresight engine (the flagship)

**Goal.** Keep the site's promise: notice the *slow* problem before it bites, by
projecting the history Joséphine already keeps.

### Engine

A new **pure** module `josephine-core/src/forecast.rs` - deterministic
least-squares linear regression, no I/O, fully unit-testable:

```rust
/// A straight-line projection of one metric toward a target value.
pub struct Forecast {
    pub slope_per_day: f64,
    pub fit_r2: f64,
    /// Days until the metric reaches `target`, when the trend is real and
    /// heading that way; `None` otherwise.
    pub eta_days: Option<f64>,
}

/// Fit a line to `(day, value)` samples and project it to `target`.
/// Returns `None` when the data can't support an honest projection.
pub fn project(points: &[(f64, f64)], target: f64, guards: &Guards) -> Option<Forecast>;
```

### Data

A new query `Storage::metric_series_since(check, metric, days)` returning
hourly-averaged points (the same downsampling as `metric_summary_24h:151`, over
N days), which bounds the row count and denoises. **No schema migration** - it
reads the existing `metrics` table.

### Targets (curated, not user-configurable at first)

| Check | Metric | Target | Meaning |
|---|---|---|---|
| disk | `usage_percent_worst` | 100 | time to a full partition |
| inode | `inode_usage_percent_worst` | 100 | time to inode exhaustion |
| memory | `usage_percent` | 100 | a leak creeping up |
| smart (if D) | `smart_wear_percent` | 100 | end of rated write life |

### Anti-noise guards (this is what keeps her calm)

A forecast is only produced - and only spoken - when **all** hold:

- at least `min_samples` points spanning a meaningful window;
- the slope points **toward** the target (a disk that is emptying says nothing);
- `fit_r2 >= min_fit` (default 0.5) - a noisy scatter stays silent;
- `eta_days <= horizon_days` (default 30) - she doesn't forecast the heat death
  of the universe.

Otherwise: nothing. Joséphine does not manufacture worry.

### Surface

A `doctor` section, printed **only when at least one forecast crosses its
horizon**, EN + FR, in the sober voice:

```
  Prévoyance
    / se remplit : plein vers vendredi (~6 j) - 91 % aujourd'hui, +1,4 %/j
```

**MVP scope, explicit:** `doctor`-only, **no notification**, **no `--json`
change** (mirrors the remedies increment). A rare "foresight" notification and a
`forecasts` array in JSON are named follow-ups, not part of this increment.

### Config

A small `forecast` block: `{ enabled, horizon_days, min_samples, min_fit }`,
with validation, defaulting to the guards above.

### Precedent

The battery check already derives a rate today (`battery_depletion_percent`), so
a forward-looking, deterministic figure is not a new kind of thing for her.

### Retention

Foresight needs several days of samples. It reads whatever window exists and
stays silent below `min_samples`, so it is safe on a fresh install; but it is
worth confirming the `history.retention_days` default is comfortably above
`horizon_days`, or the projection is fitting too short a base.

### The open decision (issue #57)

This is a plain least-squares fit: deterministic, reproducible, local, no
trained model. My reading is that it is *inside* "deterministic observation
only", not against it - but the owner drew that line and gets the call. If the
answer is no, this increment is dropped cleanly and the other five stand.

### Tests

Pure-function tests on synthetic series: a monotonic climb with a known slope ->
the expected ETA; a noisy scatter -> `None` (low R2); a flat line -> `None`; a
series moving away from the target -> `None`; a climb whose ETA is beyond the
horizon -> `None`.

---

## Increment F - weekly digest

**Goal.** A gentle, opt-in weekly summary, in the "accompagnement" spirit -
never a push she didn't invite.

- `josephine report --since 7d` (and other durations): a windowed aggregate of
  events and worst values, reusing `storage`.
- An opt-in shipped `packaging/systemd/josephine-report.timer` +
  `.service` (user units) that runs the digest weekly. Enabled by hand, like the
  main unit (`systemctl --user enable --now josephine-report.timer`).

**Files.** `commands/report_cmd.rs`, `storage.rs` (a windowed query),
`packaging/systemd/*`, and the enable instructions in the README.

---

## Cross-cutting

- Every new check (B, C, D) follows the seven-step add and **bumps the
  `all_..._checks_have_advice` guard** in `remedy.rs` - the guard rail that stops
  a check shipping without a remedy.
- Each increment refreshes `docs/CURRENT_STATE.md`, which is currently stale
  (its header still reads 0.11.0, and it calls the terminal notification channel
  "non implémenté" although 0.8.0 shipped it). A quiet correction rides along.
- Per-increment: the quality gate (`cargo fmt --check`, `cargo clippy
  --workspace --all-targets -- -D warnings`, `cargo test --workspace`) plus a
  by-eye `doctor`/`status` in both languages, and a `CHANGELOG` entry under
  `[Unreleased]`.

---

## Release

One minor per logical step, owner's preference:

- **0.13.0** - A (exit codes + `--oneline`) and B (`reboot`).
- **0.14.0** - C (PSI) and D (SSD wear).
- **0.15.0** - E (foresight) and F (weekly digest).

Or one minor per PR - whatever cadence suits the project. Nothing in here is
urgent; it is a direction, offered for the owner to accept, reshape, or decline.
