# Remedies in `doctor` — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move `josephine fix`'s remedies into `josephine doctor` as a grouped closing section backed by a new core module, then delete the `fix` command.

**Architecture:** A new `josephine-core/src/remedy.rs` becomes the single source of truth for "what to do about it". It holds the static per-check copy (`Advice`, moved out of `explain_cmd.rs`) and a `for_result` function that turns a failing `CheckResult` into ready-to-render `Remedy` values — contextual where live state allows, static otherwise. `explain` and `doctor` become pure renderings of that module; `fix_cmd.rs` disappears.

**Tech Stack:** Rust 2024 edition (rust-version 1.85), `anyhow`, `serde`, `clap`, `colored`. Workspace: `josephine-core` (logic) + `josephine` (thin CLI).

**Spec:** [`docs/superpowers/specs/2026-07-25-josephine-remedies-in-doctor-design.md`](../specs/2026-07-25-josephine-remedies-in-doctor-design.md)

## Global Constraints

- **Every user-facing string ships English AND French.** Wrap literals in `i18n::t(en, fr)`, or `match i18n::lang()` for interpolated ones. This is a `CLAUDE.md` product rule.
- **Never `ERROR` / `FATAL` / `PANIC` / `CRASH` / `ÉCHEC`** in user-facing text — enforced by the `FORBIDDEN` array in `voice.rs`'s test.
- **No remedy may refer to `josephine doctor` or `josephine fix`.** The first is circular inside doctor; the second is being deleted.
- Remedy action lines are one line, imperative, fitting **80 columns including their 4-space indent**.
- `sudo` only where genuinely required, and visible when it is.
- Core holds logic and copy; the binary crate stays a thin CLI (`CLAUDE.md`).
- Quality gate, run before every commit: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- **`doctor --json` output stays byte-identical.** No remedy joins `CheckResult`.
- Target release: **v0.11.0**.

---

## File Structure

| File | Responsibility |
|---|---|
| `crates/josephine-core/src/remedy.rs` | **New.** `Advice` (static per-check copy, 14 entries), `Remedy` (one renderable action), `advice()`, `for_result()`. |
| `crates/josephine-core/src/lib.rs` | Register `pub mod remedy;`. |
| `crates/josephine-core/src/voice.rs` | Drop `fix_tagline` / `fix_all_good`; rename `fix_hands_off` → `remedy_hands_off`; update the pool test. |
| `crates/josephine/src/commands/explain_cmd.rs` | Loses its `CheckExplanation` table; renders `remedy::advice()`. |
| `crates/josephine/src/output/doctor.rs` | Gains `print_todo_section`, called before the footer. |
| `crates/josephine/src/commands/fix_cmd.rs` | **Deleted.** |
| `crates/josephine/src/commands/mod.rs`, `src/cli.rs` | Drop every trace of the `fix` subcommand. |
| `README.md`, `docs/CURRENT_STATE.md`, `CHANGELOG.md`, `site/templates/index.html` | Documentation and site surface. |
| `Cargo.toml` | `[workspace.package] version` → `0.11.0`. |

Task order matters: `fix` is deleted (Task 4) **before** the voice pools it consumes are pruned (Task 5), so every task compiles on its own.

---

### Task 1: `remedy.rs` — the static advice table

**Files:**
- Create: `crates/josephine-core/src/remedy.rs`
- Modify: `crates/josephine-core/src/lib.rs:1-13`
- Test: inline `#[cfg(test)] mod tests` in `crates/josephine-core/src/remedy.rs`

**Interfaces:**
- Consumes: `crate::i18n` (`t`, `lang`, `Lang`).
- Produces: `pub struct Advice { pub name: &'static str, pub what: (&'static str, &'static str), pub why: (&'static str, &'static str), pub remedy: (&'static str, &'static str) }`, `pub const ADVICE: &[Advice]` (14 entries), `pub fn advice(check: &str) -> Option<&'static Advice>`, `pub fn all() -> &'static [Advice]`. Task 2 adds `Remedy` / `for_result` to this same file; Task 3 renders `advice()`.

**All fourteen remedies are rewritten.** Seven (`cpu`, `memory`, `disk`, `systemd`, `inode`, `kernel`, `filesystem`) had to change because they point back at `josephine doctor` or advertise `josephine fix`. The other seven were already actionable, but every one of them overflowed 80 columns once indented by the section's `    1. ` prefix, so they were tightened to fit. `what` and `why` are copied across unchanged for all fourteen.

The 73-character ceiling below is the 80-column budget minus that 7-character prefix, and it is enforced by a test rather than by eye.

- [ ] **Step 1: Write the failing test**

Create `crates/josephine-core/src/remedy.rs` containing only this test module for now:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_fourteen_checks_have_advice() {
        assert_eq!(ADVICE.len(), 14);
    }

    #[test]
    fn advice_is_bilingual_and_non_empty() {
        for entry in ADVICE {
            for (en, fr) in [entry.what, entry.why, entry.remedy] {
                assert!(!en.trim().is_empty(), "{} has an empty EN string", entry.name);
                assert!(!fr.trim().is_empty(), "{} has an empty FR string", entry.name);
                assert_ne!(en, fr, "{} looks untranslated", entry.name);
            }
        }
    }

    #[test]
    fn no_remedy_is_circular() {
        for entry in ADVICE {
            for text in [entry.remedy.0, entry.remedy.1] {
                assert!(
                    !text.contains("josephine doctor"),
                    "{} sends the reader back to doctor: {text}",
                    entry.name
                );
                assert!(
                    !text.contains("josephine fix"),
                    "{} references the removed fix command: {text}",
                    entry.name
                );
            }
        }
    }

    #[test]
    fn remedies_fit_a_terminal_line() {
        // 80 columns minus the section's "    1. " prefix.
        for entry in ADVICE {
            for text in [entry.remedy.0, entry.remedy.1] {
                let width = text.chars().count();
                assert!(
                    width <= 73,
                    "{} remedy is {width} chars, over the 73 budget: {text}",
                    entry.name
                );
            }
        }
    }

    #[test]
    fn advice_is_found_by_name() {
        assert_eq!(advice("cpu").unwrap().name, "cpu");
        assert!(advice("nope").is_none());
    }
}
```

`chars().count()` rather than `len()`: the French strings are accented, and UTF-8 bytes would over-count them.

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p josephine-core remedy`
Expected: FAIL — the module is not registered in `lib.rs`, and `ADVICE` / `advice` do not exist.

- [ ] **Step 3: Register the module**

In `crates/josephine-core/src/lib.rs`, add `pub mod remedy;` between `pub mod paths;` and `pub mod rules;` (the list is alphabetical):

```rust
pub mod paths;
pub mod remedy;
pub mod rules;
```

- [ ] **Step 4: Write the implementation**

Prepend to `crates/josephine-core/src/remedy.rs`, above the test module:

```rust
//! What to do about a check that came back in default.
//!
//! Single source of truth for remedies: `explain` renders the static
//! [`Advice`] copy, `doctor` renders [`Remedy`] values derived from a live
//! [`CheckResult`]. No remedy ever points back at `josephine doctor` — inside
//! doctor that would be circular.

/// Static per-check copy, independent of any run. Each field is an (EN, FR) pair.
pub struct Advice {
    pub name: &'static str,
    pub what: (&'static str, &'static str),
    pub why: (&'static str, &'static str),
    pub remedy: (&'static str, &'static str),
}

/// The fourteen checks, in the order `explain` lists them.
pub const ADVICE: &[Advice] = &[
    Advice {
        name: "cpu",
        what: (
            "Processor load and the busiest processes.",
            "Charge processeur et processus les plus actifs.",
        ),
        why: (
            "Sustained high CPU can slow everything down or point to a runaway process.",
            "Une charge CPU élevée ralentit tout ou signale un processus incontrôlé.",
        ),
        remedy: (
            "Stop the hungriest process with `kill <PID>` if it's not expected.",
            "Arrêtez le plus gourmand avec `kill <PID>`, sauf si c'est normal.",
        ),
    },
    Advice {
        name: "memory",
        what: (
            "RAM and swap usage, plus the hungriest processes.",
            "Utilisation RAM et swap, et processus les plus gourmands.",
        ),
        why: (
            "Low free memory triggers swapping and OOM kills — work slows or apps vanish.",
            "Peu de mémoire libre provoque du swap et des OOM — tout ralentit ou des apps disparaissent.",
        ),
        remedy: (
            "Close the heaviest apps first; add swap or RAM if it repeats.",
            "Fermez d'abord les apps les plus lourdes ; ajoutez swap ou RAM.",
        ),
    },
    Advice {
        name: "disk",
        what: (
            "Free space on each mounted partition.",
            "Espace libre sur chaque partition montée.",
        ),
        why: (
            "A full disk stops writes, breaks updates and can corrupt databases.",
            "Un disque plein bloque les écritures, casse les mises à jour et peut corrompre des bases.",
        ),
        remedy: (
            "`josephine clean` previews what can be freed; clear the biggest.",
            "`josephine clean` estime ce qui peut être libéré ; videz le plus gros.",
        ),
    },
    Advice {
        name: "temperature",
        what: (
            "CPU and NVMe sensor temperatures.",
            "Températures des capteurs CPU et NVMe.",
        ),
        why: (
            "Overheating throttles performance and shortens hardware life.",
            "La surchauffe limite les performances et réduit la durée de vie du matériel.",
        ),
        remedy: (
            "Check airflow and dust; `sensors` for detail. On a duvet? Move it.",
            "Vérifiez ventilation et poussière ; `sensors` pour le détail.",
        ),
    },
    Advice {
        name: "systemd",
        what: (
            "Failed units and services that restart too often.",
            "Unités en échec et services qui redémarrent trop souvent.",
        ),
        why: (
            "A failed service means something you rely on may be down; crash loops hide root causes.",
            "Un service en échec signifie qu'un composant est peut-être arrêté ; les boucles de redémarrage masquent la cause.",
        ),
        remedy: (
            "Restart it: `sudo systemctl restart <unit>`, then `systemctl status`.",
            "Relancez-la : `sudo systemctl restart <unité>`, puis `systemctl status`.",
        ),
    },
    Advice {
        name: "updates",
        what: (
            "Pending package updates (apt, dnf or pacman).",
            "Mises à jour de paquets en attente (apt, dnf ou pacman).",
        ),
        why: (
            "Unpatched packages leave known vulnerabilities and bugs on the system.",
            "Des paquets non mis à jour laissent des failles et bugs connus sur le système.",
        ),
        remedy: (
            "Apply them when convenient: `sudo apt upgrade`, `sudo dnf upgrade`.",
            "Appliquez-les quand vous pouvez : `sudo apt upgrade`, `sudo dnf upgrade`.",
        ),
    },
    Advice {
        name: "network",
        what: (
            "Round-trip latency to the default gateway.",
            "Latence aller-retour vers la passerelle par défaut.",
        ),
        why: (
            "High latency or packet loss means local network trouble before the wider internet.",
            "Une latence élevée ou des pertes signalent un souci réseau local avant Internet.",
        ),
        remedy: (
            "Check Wi-Fi signal, cables and router; `ping` the gateway to confirm.",
            "Vérifiez signal Wi-Fi, câbles et routeur ; `ping` la passerelle.",
        ),
    },
    Advice {
        name: "battery",
        what: (
            "Charge level and depletion rate on battery power.",
            "Niveau de charge et vitesse de décharge sur batterie.",
        ),
        why: (
            "A battery draining fast or stuck low means you may lose work mid-session.",
            "Une batterie qui se vide vite ou reste basse peut couper votre travail en cours.",
        ),
        remedy: (
            "Plug in if you can; check power settings and what keeps the GPU awake.",
            "Branchez si possible ; vérifiez l'alimentation et les apps gourmandes.",
        ),
    },
    Advice {
        name: "inode",
        what: (
            "Inode usage on writable filesystems.",
            "Utilisation des inodes sur les systèmes de fichiers accessibles en écriture.",
        ),
        why: (
            "A disk can be \"full\" on inodes while still showing free space — many tiny files.",
            "Un disque peut être « plein » en inodes tout en affichant de l'espace libre — beaucoup de petits fichiers.",
        ),
        remedy: (
            "Prune caches and temp trees; `du --inodes -x -d1 /` finds the nests.",
            "Purgez caches et fichiers temporaires ; `du --inodes -x -d1 /` aide.",
        ),
    },
    Advice {
        name: "smart",
        what: (
            "SMART health status of disks (opt-in, often needs root).",
            "État de santé SMART des disques (opt-in, souvent root requis).",
        ),
        why: (
            "SMART warnings often precede hard drive failure by days or weeks.",
            "Les alertes SMART précèdent souvent une panne disque de quelques jours ou semaines.",
        ),
        remedy: (
            "Back up immediately; enable the check in config if `smartctl` allows.",
            "Sauvegardez immédiatement ; activez le check si `smartctl` le permet.",
        ),
    },
    Advice {
        name: "kernel",
        what: (
            "Kernel incidents in the last hour (OOM kills, oops, panics).",
            "Incidents noyau sur la dernière heure (OOM, oops, panics).",
        ),
        why: (
            "Kernel faults destabilise the whole machine — not just one app.",
            "Les fautes noyau déstabilisent toute la machine — pas seulement une app.",
        ),
        remedy: (
            "Read it: `journalctl -k -p err --since -1h`; back up if it repeats.",
            "Lisez : `journalctl -k -p err --since -1h` ; sauvegardez si ça répète.",
        ),
    },
    Advice {
        name: "filesystem",
        what: (
            "Writable filesystems unexpectedly mounted read-only.",
            "Systèmes de fichiers habituellement accessibles en écriture montés en lecture seule.",
        ),
        why: (
            "A silent read-only remount often means disk errors or corruption — data loss risk.",
            "Un remontage silencieux en lecture seule signale souvent erreurs disque ou corruption — risque de perte.",
        ),
        remedy: (
            "Back up what matters now, then `dmesg -T | grep -i read-only`.",
            "Sauvegardez ce qui compte, puis `dmesg -T | grep -i read-only`.",
        ),
    },
    Advice {
        name: "timesync",
        what: (
            "Whether the system clock is synchronised via NTP.",
            "Si l'horloge système est synchronisée via NTP.",
        ),
        why: (
            "Clock drift breaks log ordering, TLS validation and scheduled jobs.",
            "Une horloge qui dérive casse l'ordre des journaux, la validation TLS et les tâches planifiées.",
        ),
        remedy: (
            "`timedatectl set-ntp true` usually fixes it; check `timedatectl`.",
            "`timedatectl set-ntp true` règle souvent ça ; voyez `timedatectl`.",
        ),
    },
    Advice {
        name: "security",
        what: (
            "Failed login and authentication attempts in the last hour.",
            "Tentatives de connexion et d'authentification échouées sur la dernière heure.",
        ),
        why: (
            "Bursts of failed logins may mean someone is probing your machine.",
            "Des rafales de connexions échouées peuvent signifier qu'on sonde votre machine.",
        ),
        remedy: (
            "Review `journalctl -u sshd`; keys-only SSH and fail2ban if not you.",
            "Voyez `journalctl -u sshd` ; SSH par clés et fail2ban si pas vous.",
        ),
    },
];

/// The advice for one check, by its internal name (`cpu`, `disk`, …).
pub fn advice(check: &str) -> Option<&'static Advice> {
    ADVICE.iter().find(|a| a.name == check)
}

/// Every check's advice, in listing order.
pub fn all() -> &'static [Advice] {
    ADVICE
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p josephine-core remedy`
Expected: PASS — 4 tests.

- [ ] **Step 6: Run the quality gate**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: all green. `explain_cmd.rs` still has its own copy of the table — that duplication is removed in Task 3.

- [ ] **Step 7: Commit**

```bash
git add crates/josephine-core/src/remedy.rs crates/josephine-core/src/lib.rs
git commit -m "feat(core): remedy module with the fourteen checks' advice

Seven remedies rewritten to stop pointing back at josephine doctor (or
at the fix command that is about to disappear); the other seven move
across verbatim."
```

---

### Task 2: `remedy::for_result` — contextual remedies

**Files:**
- Modify: `crates/josephine-core/src/remedy.rs`
- Test: inline `#[cfg(test)] mod tests` in the same file

**Interfaces:**
- Consumes: `Advice` / `advice()` from Task 1; `crate::check::{CheckResult, Severity}`; `crate::i18n`.
- Produces: `pub struct Remedy { pub action: String, pub hint: Option<String> }`, `pub fn for_result(result: &CheckResult) -> Vec<Remedy>`, `pub const MAX_ENTRIES_PER_CHECK: usize = 5`. Task 5 renders these in `doctor.rs`.

Rules: an `Info` result yields nothing. `systemd` yields one entry per failed unit (from `top_processes`), capped at `MAX_ENTRIES_PER_CHECK` with a counted overflow line. `cpu` and `memory` quote `top_processes[0]` verbatim. Everything else falls back to the static `advice().remedy`.

- [ ] **Step 1: Write the failing tests**

Append to the `mod tests` block in `crates/josephine-core/src/remedy.rs`:

```rust
    use crate::check::{CheckResult, Metric, Severity};

    /// A `CheckResult` whose single metric lands on the given severity.
    fn result(name: &str, severity: Severity, top: &[&str]) -> CheckResult {
        let value = match severity {
            Severity::Info => 0.0,
            Severity::Attention => 80.0,
            Severity::Critique => 95.0,
        };
        CheckResult {
            check_name: name.into(),
            metrics: vec![Metric {
                name: "usage_percent".into(),
                value,
                unit: "%".into(),
                threshold_warning: Some(75.0),
                threshold_critical: Some(90.0),
            }],
            details: Vec::new(),
            top_processes: top.iter().map(|s| s.to_string()).collect(),
            status_value: None,
        }
    }

    #[test]
    fn healthy_check_yields_no_remedy() {
        assert!(for_result(&result("disk", Severity::Info, &[])).is_empty());
    }

    #[test]
    fn systemd_yields_one_entry_per_failed_unit() {
        let r = result(
            "systemd",
            Severity::Critique,
            &["nginx.service", "backup.timer"],
        );
        let remedies = for_result(&r);
        assert_eq!(remedies.len(), 2);
        assert_eq!(remedies[0].action, "sudo systemctl restart nginx.service");
        assert_eq!(remedies[1].action, "sudo systemctl restart backup.timer");
        assert!(
            remedies[0]
                .hint
                .as_ref()
                .unwrap()
                .contains("systemctl status nginx.service")
        );
    }

    #[test]
    fn systemd_caps_the_list_and_counts_the_overflow() {
        let units = [
            "a.service", "b.service", "c.service", "d.service", "e.service", "f.service",
            "g.service",
        ];
        let remedies = for_result(&result("systemd", Severity::Critique, &units));
        assert_eq!(remedies.len(), MAX_ENTRIES_PER_CHECK + 1);
        let overflow = &remedies[MAX_ENTRIES_PER_CHECK].action;
        assert!(overflow.contains('2'), "overflow must count the rest: {overflow}");
        assert!(remedies[MAX_ENTRIES_PER_CHECK].hint.is_none());
    }

    #[test]
    fn cpu_quotes_the_top_process_verbatim() {
        let r = result("cpu", Severity::Attention, &["firefox (PID 1234) — 87.3 %"]);
        let remedies = for_result(&r);
        assert_eq!(remedies.len(), 1);
        assert!(
            remedies[0].action.contains("firefox (PID 1234) — 87.3 %"),
            "got: {}",
            remedies[0].action
        );
        assert!(remedies[0].hint.is_some());
    }

    #[test]
    fn cpu_without_process_list_falls_back_to_static_advice() {
        let remedies = for_result(&result("cpu", Severity::Attention, &[]));
        assert_eq!(remedies.len(), 1);
        assert_eq!(remedies[0].action, advice("cpu").unwrap().remedy.0);
    }

    #[test]
    fn check_without_contextual_rule_falls_back_to_static_advice() {
        let remedies = for_result(&result("timesync", Severity::Attention, &[]));
        assert_eq!(remedies.len(), 1);
        assert_eq!(remedies[0].action, advice("timesync").unwrap().remedy.0);
        assert!(remedies[0].hint.is_none());
    }

    #[test]
    fn unknown_check_yields_no_remedy() {
        assert!(for_result(&result("nope", Severity::Critique, &[])).is_empty());
    }
```

The two fallback tests compare against `remedy.0` (the English string) and therefore assume the default language. Add this to the top of `mod tests` so the suite is order-independent:

```rust
    use crate::i18n::{self, Lang};

    /// The fallback tests read the EN string, so pin the language.
    fn with_english<T>(f: impl FnOnce() -> T) -> T {
        let prev = i18n::lang();
        i18n::set_lang(Lang::En);
        let out = f();
        i18n::set_lang(prev);
        out
    }
```

and wrap the two fallback assertions, e.g.:

```rust
    #[test]
    fn check_without_contextual_rule_falls_back_to_static_advice() {
        with_english(|| {
            let remedies = for_result(&result("timesync", Severity::Attention, &[]));
            assert_eq!(remedies.len(), 1);
            assert_eq!(remedies[0].action, advice("timesync").unwrap().remedy.0);
            assert!(remedies[0].hint.is_none());
        });
    }
```

Apply the same wrapper to `cpu_without_process_list_falls_back_to_static_advice`.

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p josephine-core remedy`
Expected: FAIL to compile — `Remedy`, `for_result` and `MAX_ENTRIES_PER_CHECK` do not exist yet. Task 1's four tests are in the same module, so the whole module fails to build; that is expected and resolves in Step 3.

- [ ] **Step 3: Write the implementation**

Add to `crates/josephine-core/src/remedy.rs`, after `pub fn all()`:

```rust
use crate::check::{CheckResult, Severity};
use crate::i18n::{self, Lang};

/// At most this many entries per check in the "what's left to do" section;
/// beyond it, a counted overflow line. Nothing is dropped silently.
pub const MAX_ENTRIES_PER_CHECK: usize = 5;

/// One actionable step, ready to render.
pub struct Remedy {
    /// The action — usually a command to run.
    pub action: String,
    /// Optional second line: how to understand the problem before acting.
    pub hint: Option<String>,
}

impl Remedy {
    fn action(action: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            hint: None,
        }
    }

    fn with_hint(action: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            hint: Some(hint.into()),
        }
    }
}

/// The actions for a check that came back Attention or Critique.
///
/// Contextual where the collected state allows it (`systemd`, `cpu`,
/// `memory`), static [`Advice`] copy otherwise. A healthy check yields
/// nothing — Joséphine does not manufacture work.
pub fn for_result(result: &CheckResult) -> Vec<Remedy> {
    if result.worst_severity() == Severity::Info {
        return Vec::new();
    }
    let Some(advice) = advice(&result.check_name) else {
        return Vec::new();
    };

    match result.check_name.as_str() {
        "systemd" if !result.top_processes.is_empty() => failed_units(&result.top_processes),
        "cpu" | "memory" => match result.top_processes.first() {
            Some(top) => vec![Remedy::with_hint(
                match i18n::lang() {
                    Lang::En => format!("Biggest consumer right now: {top}"),
                    Lang::Fr => format!("Le plus gourmand en ce moment : {top}"),
                },
                i18n::t(advice.remedy.0, advice.remedy.1),
            )],
            None => vec![Remedy::action(i18n::t(advice.remedy.0, advice.remedy.1))],
        },
        _ => vec![Remedy::action(i18n::t(advice.remedy.0, advice.remedy.1))],
    }
}

/// One restart per failed unit, capped, with a counted overflow line.
fn failed_units(units: &[String]) -> Vec<Remedy> {
    let mut remedies: Vec<Remedy> = units
        .iter()
        .take(MAX_ENTRIES_PER_CHECK)
        .map(|unit| {
            Remedy::with_hint(
                format!("sudo systemctl restart {unit}"),
                match i18n::lang() {
                    Lang::En => format!("(find out why: systemctl status {unit})"),
                    Lang::Fr => format!("(comprendre pourquoi : systemctl status {unit})"),
                },
            )
        })
        .collect();

    let hidden = units.len().saturating_sub(MAX_ENTRIES_PER_CHECK);
    if hidden > 0 {
        remedies.push(Remedy::action(match i18n::lang() {
            Lang::En => format!("+ {hidden} more failed units — `systemctl --failed` lists them"),
            Lang::Fr => {
                format!("+ {hidden} autres unités en échec — `systemctl --failed` les liste")
            }
        }));
    }
    remedies
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p josephine-core remedy`
Expected: PASS — 11 tests.

- [ ] **Step 5: Run the quality gate**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: all green.

- [ ] **Step 6: Commit**

```bash
git add crates/josephine-core/src/remedy.rs
git commit -m "feat(core): derive remedies from a live check result

Contextual for systemd (one restart per failed unit, capped at five with
a counted overflow), cpu and memory (top consumer quoted verbatim);
static advice everywhere else. A healthy check yields nothing."
```

---

### Task 3: `explain` renders the core table

**Files:**
- Modify: `crates/josephine/src/commands/explain_cmd.rs:1-308` (the `CheckExplanation` struct and its 14-entry `CHECKS` table are deleted; the four print functions are rewired)
- Test: the existing `all_fourteen_checks_are_listed` test moves to core (already covered by Task 1's `all_fourteen_checks_have_advice`), so the local test module is removed.

**Interfaces:**
- Consumes: `josephine_core::remedy::{Advice, advice, all}` from Task 1.
- Produces: no new interface. `explain`'s rendered output is unchanged.

- [ ] **Step 1: Verify the current output, to compare against**

Run: `cargo run -p josephine -- explain > /tmp/explain-before.txt && cargo run -p josephine -- explain systemd >> /tmp/explain-before.txt && cargo run -p josephine -- explain nope >> /tmp/explain-before.txt`
Expected: the file captures the list, one detail view, and the unknown-check view.

Note: `explain systemd`'s remedy line **will** differ afterwards — it currently advertises `josephine fix`. That single line is the intended change; everything else must match.

- [ ] **Step 2: Rewrite the command as a rendering layer**

Replace the whole of `crates/josephine/src/commands/explain_cmd.rs` with:

```rust
//! `josephine explain` — what each check watches, why it matters, and how to act.
//!
//! Pure rendering: the copy lives in `josephine_core::remedy`, shared with the
//! remedies `doctor` prints.

use anyhow::Result;
use josephine_core::i18n::{self, Lang};
use josephine_core::remedy::{Advice, advice, all};

use crate::output::{check_label, sober_header};

pub fn run(check: Option<&str>) -> Result<()> {
    sober_header(Some(i18n::t("explain", "explain")), None);

    match check {
        None => print_list(),
        Some(name) => match advice(name) {
            Some(entry) => print_detail(entry),
            None => print_unknown(name),
        },
    }

    Ok(())
}

fn print_list() {
    println!(
        "{}",
        i18n::t(
            "What Joséphine watches — one line each. Detail: `josephine explain <check>`.",
            "Ce que Joséphine surveille — une ligne chacun. Détail : `josephine explain <check>`.",
        )
    );
    println!();
    for entry in all() {
        let label = check_label(entry.name);
        let what = i18n::t(entry.what.0, entry.what.1);
        println!("  {label} ({}) — {what}", entry.name);
    }
}

fn print_detail(entry: &Advice) {
    let label = check_label(entry.name);
    println!("{label} ({})", entry.name);
    println!();
    println!(
        "{} {}",
        i18n::t("What:", "Quoi :"),
        i18n::t(entry.what.0, entry.what.1)
    );
    println!(
        "{} {}",
        i18n::t("Why:", "Pourquoi :"),
        i18n::t(entry.why.0, entry.why.1)
    );
    println!(
        "{} {}",
        i18n::t("Remedy:", "Remède :"),
        i18n::t(entry.remedy.0, entry.remedy.1)
    );
}

fn print_unknown(name: &str) {
    let names: Vec<&str> = all().iter().map(|a| a.name).collect();
    match i18n::lang() {
        Lang::En => {
            println!("Unknown check \"{name}\". Known checks:");
            for n in &names {
                println!("  {n}");
            }
        }
        Lang::Fr => {
            println!("Check inconnu « {name} ». Checks connus :");
            for n in &names {
                println!("  {n}");
            }
        }
    }
}
```

- [ ] **Step 3: Run the tests**

Run: `cargo test --workspace`
Expected: PASS. The `all_fourteen_checks_are_listed` test disappeared with the local table; `all_fourteen_checks_have_advice` in core covers it.

- [ ] **Step 4: Diff the rendered output**

Run: `cargo run -p josephine -- explain > /tmp/explain-after.txt && cargo run -p josephine -- explain systemd >> /tmp/explain-after.txt && cargo run -p josephine -- explain nope >> /tmp/explain-after.txt && diff /tmp/explain-before.txt /tmp/explain-after.txt`
Expected: the only differences are the seven rewritten remedy lines — and in this capture, only the `systemd` one (the list view shows `what`, not `remedy`).

- [ ] **Step 5: Check the French side too**

There is **no environment override** for the language — `i18n` exposes only `set_lang`, `lang` and `t`, and the value comes from `~/.config/josephine/config.yaml`. Use a throwaway `HOME`, exactly as the integration tests do:

```bash
TMPHOME=$(mktemp -d)
env -u XDG_CONFIG_HOME -u XDG_DATA_HOME HOME="$TMPHOME" \
  cargo run -q -p josephine -- config show > /dev/null   # writes the default config
sed -i 's/^language: .*/language: fr/' "$TMPHOME/.config/josephine/config.yaml"
env -u XDG_CONFIG_HOME -u XDG_DATA_HOME HOME="$TMPHOME" \
  cargo run -q -p josephine -- explain kernel
```

Expected: the rewritten French remedy renders, accented and correct. Keep `$TMPHOME` around — Task 5 reuses it.

- [ ] **Step 6: Run the quality gate**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: all green.

- [ ] **Step 7: Commit**

```bash
git add crates/josephine/src/commands/explain_cmd.rs
git commit -m "refactor(explain): render the shared core advice table

The fourteen entries now live in josephine-core::remedy, so explain and
doctor cannot drift apart."
```

---

### Task 4: Delete the `fix` command

**Files:**
- Delete: `crates/josephine/src/commands/fix_cmd.rs`
- Modify: `crates/josephine/src/commands/mod.rs:6`
- Modify: `crates/josephine/src/cli.rs:7-10, 61-62, 123-125, 178`

**Interfaces:**
- Consumes: nothing new.
- Produces: the `Fix` variant no longer exists on the `Commands` enum. `voice::fix_*` are left in place until Task 5 so the crate keeps compiling at every step.

The two `parse_failed_units` unit tests are deleted with the file. Core keeps its own systemd parsing and tests in `checks/systemd.rs`, and the second `systemctl --failed` invocation this file performed disappears entirely.

`crates/josephine/tests/cli.rs` contains **no** `fix` test today, so nothing there breaks. (`docs/CURRENT_STATE.md:163` claims integration tests cover "le parsing des commandes (`clean`, `fix`)" — that claim is already stale; Task 6 corrects it.) Step 1 below adds the test that pins the removal.

- [ ] **Step 1: Write the failing test**

Add to `crates/josephine/tests/cli.rs`, after `unknown_command_fails`:

```rust
#[test]
fn fix_is_no_longer_a_subcommand() {
    Command::cargo_bin("josephine")
        .unwrap()
        .arg("fix")
        .assert()
        .failure();
}

#[test]
fn help_does_not_offer_fix() {
    Command::cargo_bin("josephine")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("fix").not());
}
```

`.not()` needs the trait in scope — extend the import at the top of the file:

```rust
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p josephine --test cli fix`
Expected: both FAIL — `fix` is still a working subcommand and still appears in `--help`.

- [ ] **Step 3: Delete the command module**

```bash
git rm crates/josephine/src/commands/fix_cmd.rs
```

- [ ] **Step 2: Drop the module declaration**

In `crates/josephine/src/commands/mod.rs`, remove line 6:

```rust
pub mod fix_cmd;
```

- [ ] **Step 3: Drop the import in `cli.rs`**

Change the `use` block at `crates/josephine/src/cli.rs:7-10` to:

```rust
use crate::commands::{
    ConfigAction, DaemonAction, NotifyAction, clean_cmd, config_cmd, daemon_cmd, doctor_cmd,
    explain_cmd, history_cmd, notify_cmd, report_cmd, status_cmd, update_cmd,
};
```

- [ ] **Step 4: Drop the subcommand variant**

Remove these two lines from the `Commands` enum (`crates/josephine/src/cli.rs:61-62`):

```rust
    /// Guided fixes: what's wrong and how to set it right — her finger-snap
    Fix,
```

- [ ] **Step 5: Drop the French help entry**

Remove this from `localize_help_fr` (`crates/josephine/src/cli.rs:123-125`):

```rust
        .mut_subcommand("fix", |c| {
            c.about("Réparations guidées : ce qui cloche et comment y remédier — son claquement de doigts")
        })
```

- [ ] **Step 6: Drop the dispatch arm**

Remove this from the `match cli.command` block (`crates/josephine/src/cli.rs:178`):

```rust
        Some(Commands::Fix) => fix_cmd::run()?,
```

- [ ] **Step 7: Verify the command is gone**

Run: `cargo run -p josephine -- fix`
Expected: clap rejects it — `error: unrecognized subcommand 'fix'`.

Run: `cargo run -p josephine -- --help`
Expected: no `fix` line. `cargo run -p josephine -- completions bash | grep -c fix` returns `0`.

- [ ] **Step 8: Run the quality gate**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: all green. `voice::fix_*` are `pub` in a library crate, so no dead-code warning until Task 5 removes them.

- [ ] **Step 9: Commit**

```bash
git add -A crates/josephine/src
git commit -m "feat(cli)!: remove the fix command

It promised a repair it explicitly refused to perform, and covered only
two of the fourteen checks. Its remedies now live in doctor."
```

---

### Task 5: The "what's left to do" section in `doctor`

**Files:**
- Modify: `crates/josephine/src/output/doctor.rs:1-34`
- Modify: `crates/josephine-core/src/voice.rs:146-200, 355-365`
- Test: inline, in `crates/josephine-core/src/voice.rs`

**Interfaces:**
- Consumes: `remedy::{Remedy, for_result}` from Task 2; `voice::remedy_hands_off`.
- Produces: `fn print_todo_section(results: &[CheckResult])`, private to `doctor.rs`. `voice::remedy_hands_off()` replaces `voice::fix_hands_off()`.

- [ ] **Step 1: Rename the surviving voice pool**

In `crates/josephine-core/src/voice.rs`, replace the section header comment at line 146 and rename the function at line 186:

```rust
// --- remedies (her finger-snap) ---------------------------------------------

/// Closing line for the "what's left to do" section — she guides, you act.
pub fn remedy_hands_off() -> &'static str {
    pick(&[
        (
            "✧ No magic wand here: I show the way, you keep the wheel.",
            "✧ Pas de baguette magique ici : je montre le chemin, vous gardez le volant.",
        ),
        (
            "✧ I point, you press. Nothing runs behind your back.",
            "✧ Je montre, vous appuyez. Rien ne s'exécute dans votre dos.",
        ),
        (
            "✧ The wheel stays yours — I just lean over your shoulder.",
            "✧ Le volant reste à vous — je me contente de regarder par-dessus votre épaule.",
        ),
    ])
}
```

- [ ] **Step 2: Delete the two orphaned pools**

Remove `pub fn fix_tagline()` (lines 148-165) and `pub fn fix_all_good()` (lines 167-183) entirely. `doctor` already opens on its own verdict and carries its own all-clear line, so both would duplicate existing copy.

- [ ] **Step 3: Update the pool test**

In `crates/josephine-core/src/voice.rs`, replace the three `fix_*` pushes (lines 360-362) with one:

```rust
                all.push(remedy_hands_off());
```

- [ ] **Step 4: Run the voice tests to verify they pass**

Run: `cargo test -p josephine-core voice`
Expected: PASS — `every_pool_is_bilingual_and_calm` still green with the renamed pool.

- [ ] **Step 5: Write the section renderer**

In `crates/josephine/src/output/doctor.rs`, add the import at the top:

```rust
use josephine_core::remedy::{self, Remedy};
```

and add this function after `print_check_block`:

```rust
/// The grouped closing section: every action left to take, most severe first.
///
/// Prints nothing when no check is in default — a healthy machine gets no
/// to-do list.
fn print_todo_section(results: &[CheckResult]) {
    let mut failing: Vec<&CheckResult> = results
        .iter()
        .filter(|r| r.worst_severity() != Severity::Info)
        .collect();
    // Stable sort: Critique before Attention, insertion order kept within a band.
    failing.sort_by(|a, b| b.worst_severity().cmp(&a.worst_severity()));

    let remedies: Vec<Remedy> = failing
        .iter()
        .flat_map(|r| remedy::for_result(r))
        .collect();
    if remedies.is_empty() {
        return;
    }

    println!();
    println!(
        "  {}",
        i18n::t("What's left to do", "Ce qu'il reste à faire")
    );
    for (i, entry) in remedies.iter().enumerate() {
        println!("    {}. {}", i + 1, entry.action);
        if let Some(hint) = &entry.hint {
            println!("       {hint}");
        }
    }
    println!();
    println!(" {}", voice::remedy_hands_off());
}
```

- [ ] **Step 6: Call it from `print_doctor`**

In `print_doctor` (`crates/josephine/src/output/doctor.rs:11-34`), insert the call between the check blocks and the footer:

```rust
    for result in results {
        print_check_block(result, config, verbose);
    }
    print_todo_section(results);
    println!();
    super::style::sober_footer(footer_hint(verbose));
```

- [ ] **Step 7: See it on a real machine**

Run: `cargo run -p josephine -- doctor`
Expected: on a healthy machine, no section at all and the report ends exactly as before. To force the section, temporarily lower a threshold in the config (e.g. `disk.warning: 1`) and re-run — the section must list `josephine clean`, then the hands-off line, then the `--verbose` footer. Restore the config afterwards.

Run: `cargo run -p josephine -- doctor --json | head -5`
Expected: unchanged JSON — no `remedy` key anywhere. Confirm with `cargo run -p josephine -- doctor --json | grep -c remedy` returning `0`.

- [ ] **Step 8: Check the French rendering**

Reuse the throwaway `HOME` from Task 3 Step 5 (or rebuild it the same way), lower a threshold in `$TMPHOME/.config/josephine/config.yaml`, and run:

```bash
env -u XDG_CONFIG_HOME -u XDG_DATA_HOME HOME="$TMPHOME" \
  cargo run -q -p josephine -- doctor
```

Expected: `Ce qu'il reste à faire`, accented correctly, no `ERROR`/`ÉCHEC` vocabulary, and the hands-off line in French.

- [ ] **Step 9: Run the quality gate**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
Expected: all green.

- [ ] **Step 10: Commit**

```bash
git add crates/josephine/src/output/doctor.rs crates/josephine-core/src/voice.rs
git commit -m "feat(doctor): close the report with what's left to do

Actions are grouped after the check blocks, most severe first, and only
when something needs doing. fix's hands-off closing line survives the
command as voice::remedy_hands_off."
```

---

### Task 6: Documentation and site

**Files:**
- Modify: `README.md:119`
- Modify: `docs/CURRENT_STATE.md:48, 74, 86, 91, 163`
- Modify: `site/templates/index.html:154`
- Modify: `CHANGELOG.md:8` (the `[Unreleased]` section)

**Interfaces:** none — prose and markup only.

`docs/ROADMAP.md:61` is deliberately **left untouched**: it records what shipped in v0.4, and editing history would falsify the record.

- [ ] **Step 1: Drop `fix` from the README usage block**

Remove line 119 of `README.md`:

```
josephine fix           # guided remediation for failed services / low disk
```

Check the surrounding block for a `doctor` line; if it does not already mention the remedies, extend its comment to `# full diagnostics, and what's left to do`.

- [ ] **Step 2: Update `docs/CURRENT_STATE.md`**

Five mentions to treat, at lines 48, 74, 86, 91 and 163. Line 48 is a command table — drop `fix` from the cell. Lines 74, 86, 91 and 163 are prose about the tone pass and the commands it covered; rewrite them so they describe the current surface without claiming `fix` exists. Re-read each line in context rather than pattern-replacing: they are French prose, and a blind substitution will produce broken sentences.

- [ ] **Step 3: Rewrite the site's critical notification**

In `site/templates/index.html:150-155`, the demo toast promises exactly what the product does not do. Replace the text and command:

```html
        <p class="toast__text">{% if fr %}En échec depuis 14:02. Joséphine vous montre la commande pour le relancer :{% else %}Failed since 14:02. Joséphine shows you the command to bring it back:{% endif %}</p>
        <code class="toast__cmd">josephine doctor</code>
```

- [ ] **Step 4: Show the new section in the CLI showcase**

Find the block in `site/templates/index.html` that renders sample `doctor` output (search for `doctor` in the template and in `site/sass/main.scss` for the matching class). Add the closing section to that sample, in both languages, matching the real rendering:

```
  Ce qu'il reste à faire
    1. sudo systemctl restart nginx.service
       (comprendre pourquoi : systemctl status nginx.service)
    2. josephine clean — voyez ce qui peut être libéré
```

If no `doctor` sample exists in the template, skip this step and note it in the commit message rather than inventing a new section.

- [ ] **Step 5: Build the site to verify**

Run: `cd site && zola build`
Expected: builds clean. If `zola` is not installed, run `zola check` or skip with a note — do not guess at template syntax you cannot verify.

- [ ] **Step 6: Write the changelog entry**

Under `## [Unreleased]` in `CHANGELOG.md`, add:

```markdown
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

- Seven of the fourteen remedies rewritten so none sends you back to
  `josephine doctor` — inside doctor that was circular.
```

- [ ] **Step 7: Verify no stale reference survives**

Run: `grep -rn "josephine fix" --include="*.rs" --include="*.md" --include="*.html" --include="*.toml" . | grep -v "^./target" | grep -v CHANGELOG.md | grep -v "docs/ROADMAP.md" | grep -v "docs/superpowers"`
Expected: no output. Hits in `CHANGELOG.md`, `docs/ROADMAP.md` and `docs/superpowers/` are intentional historical records.

- [ ] **Step 8: Commit**

```bash
git add README.md docs/CURRENT_STATE.md site/templates/index.html CHANGELOG.md
git commit -m "docs+site: fix is gone, doctor carries the remedies

The site's critical notification promised guided remediation could
restart the service; it now says what actually happens."
```

---

### Task 7: Release v0.11.0

**Files:**
- Modify: `Cargo.toml:6`
- Modify: `CHANGELOG.md:8`
- Modify: `docs/CURRENT_STATE.md` (version baseline header, if it carries one)

**Interfaces:** none.

Both crates inherit `version.workspace = true`, so the root `Cargo.toml` is the only version to bump.

- [ ] **Step 1: Bump the workspace version**

In `Cargo.toml`, change line 6:

```toml
version = "0.11.0"
```

- [ ] **Step 2: Close the changelog section**

Replace `## [Unreleased]` with `## [Unreleased]` followed by a blank line and `## [0.11.0] - 2026-07-25`, moving the Added/Removed/Changed blocks written in Task 6 under the dated heading.

- [ ] **Step 3: Refresh the lockfile**

Run: `cargo check --workspace`
Expected: `Cargo.lock` picks up `0.11.0` for both crates.

- [ ] **Step 4: Run the full quality gate**

Run: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo run -p josephine -- status`
Expected: all green, and `status` renders normally.

- [ ] **Step 5: Walk the whole CLI once, by eye**

Run each of: `cargo run -p josephine -- doctor`, `doctor --verbose`, `doctor --json`, `explain`, `explain systemd`, `status`, `history`.
Expected: no mention of `fix` anywhere; `doctor` closes on the new section when something is in default; JSON unchanged.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md docs/CURRENT_STATE.md
git commit -m "release: v0.11.0"
```

---

## Deferred, deliberately

- **`doctor --json` does not expose remedies.** Adding a `remedies` array to the JSON contract is a candidate follow-up; it was ruled out here to keep the increment to one clean change.
- **The two images in `site/static/`** (`hero.png`, `why.png`) are untracked and out of scope. Three issues are on record: `why.png` renders garbled text in its mock dashboard ("TEMPERMIVEE", "Teintsservice", "PLAIES"), the pair weighs 4.6 MB, and a chibi mascot is a change of visual register for a site built on sober CLI output. They need their own decision.
