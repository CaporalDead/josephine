# Joséphine — Remedies move into `doctor`, `fix` is removed

**Date:** 2026-07-25
**Status:** Design approved, spec under review
**Scope:** CLI surface — delete `josephine fix`, fold its remedies into `doctor`
**Target release:** v0.11.0
**Branch:** `feat/remedies-in-doctor`

---

## Context

`josephine fix` promises a repair the code deliberately refuses to perform. Its
own module doc says so (`commands/fix_cmd.rs:1-3`): *"She points the way; you
keep the wheel — nothing privileged runs on its own."* Its closing line says it
again (`voice.rs:184-186`): *"I point, you press. Nothing runs behind your
back."* The command name is the only part of the feature that claims otherwise.

The mismatch is not only in the name. `fix` covers **two subjects out of
fourteen checks** — failed systemd units and disk pressure — while `explain`
covers all fourteen and `doctor` runs all fourteen. A user whose temperature or
filesystem check goes critical gets nothing from `fix`.

A sibling question was settled first: `doctor` **keeps** its name. v0.10.0
(commit `ecd0d95`) already answered the "shouldn't this be `inspect`?" objection
by making `doctor` open on a real verdict rather than renaming it, and the CLI
idiom (`brew doctor`, `flutter doctor`, `npm doctor`) diagnoses without
repairing. A doctor diagnoses and prescribes; the surgeon operates.

### Decisions locked during brainstorming

- **Direction:** absorb `fix` into `doctor`, then delete the command.
- **Placement:** one grouped section closing the report, not remedies scattered
  under each check block.
- **Fate of `fix`:** clean removal — no alias, no deprecation shim.
- **Coverage:** all fourteen remedies rewritten to be actionable and
  non-circular, shared by `explain` and `doctor`.
- **Structure:** a new `remedy.rs` in `josephine-core`, per the CLAUDE.md rule
  that core holds pure logic and the binary stays a thin CLI.

---

## Architecture

A new module `crates/josephine-core/src/remedy.rs` becomes the single source of
truth for "what to do about it".

```rust
/// One actionable step, ready to render.
pub struct Remedy {
    /// The action — usually a command to run.
    pub action: String,
    /// Optional second line: how to understand the problem before acting.
    pub hint: Option<String>,
}

/// Static per-check copy, independent of any run. Each field is an (EN, FR) pair.
pub struct Advice {
    pub name: &'static str,
    pub what: (&'static str, &'static str),
    pub why: (&'static str, &'static str),
    pub remedy: (&'static str, &'static str),
}

/// Static what/why/remedy copy for a check, independent of any run.
pub fn advice(check: &str) -> Option<&'static Advice>;

/// Actions for a check that came back Attention or Critique.
/// Contextual where live state allows it, static fallback otherwise.
pub fn for_result(result: &CheckResult) -> Vec<Remedy>;
```

The `CheckExplanation` table currently living in `commands/explain_cmd.rs` moves
to core as `Advice` (`what` / `why` / `remedy`, EN + FR). Both commands become
pure rendering: `explain` prints `advice()`, `doctor` prints `for_result()`.

**No check is re-run and no external process is spawned.** `fix_cmd.rs` shells
out to `systemctl --failed` a second time today; that call disappears, because
`checks/systemd.rs:105` already carries the failed unit names in the
`CheckResult` (`top_processes: snapshot.failed_units.clone()`), and `doctor`
renders `top_processes` only for `cpu` and `memory`, so the data is present and
currently unused.

### Data flow

`doctor_cmd::run` → `run_checks_with_progress` → for each result whose
`worst_severity() != Severity::Info`, `remedy::for_result` → aggregated list →
rendered as the closing section.

Ordering is deterministic: `Critique` before `Attention`, and within equal
severity the order in which `run_all_checks` returns the results (a stable sort,
so no reordering happens inside a severity band).

**Overflow cap.** `systemd` emits one entry per failed unit, which on a badly
broken machine could flood the report. The section prints at most **five**
entries per check; beyond that it closes with a counted overflow line
(`+ 3 autres unités en échec` / `+ 3 more failed units`). Nothing is silently
dropped — the count is always shown.

---

## Rendering

The section appears **only when at least one check is in default**. On a healthy
machine nothing is printed: the existing verdict and footer carry the report.
Joséphine does not manufacture work.

```
✦ diagnostic
  Deux ou trois choses à regarder, rien d'alarmant.
  14 contrôles · 2 à regarder

 ●  CPU · ok
    ▁▁▂▁  12 %
 ▲  Disque · attention
    ███████▁  91 %
    /home 91 % · 4,2 Gio libres
 ✕  Services · critique
    nginx.service en échec

  Ce qu'il reste à faire
    1. sudo systemctl restart nginx.service
       (comprendre pourquoi : systemctl status nginx.service)
    2. `josephine clean` estime ce qui peut être libéré ; videz le plus gros.

 ✧ Je montre, vous appuyez. Rien ne s'exécute dans votre dos.
```

Section heading: `What's left to do` / `Ce qu'il reste à faire`. It is structural,
not flavour, so it does **not** rotate through a voice pool.

### Saving the voice

`voice.rs` holds three pools written for `fix` — nine EN + FR phrasings that
would die with the command. `fix_hands_off` is **kept and renamed
`remedy_hands_off`**: it closes the new section and states precisely the thing
the removal of `fix` might otherwise seem to take away. `fix_tagline` and
`fix_all_good` are dropped — `doctor` already has its own opening verdict and
its own all-clear line, and keeping them would duplicate both.

When actions exist the report therefore ends on two distinct lines: the
hands-off closing note, then the existing `sober_footer` pointing at
`--verbose`. They say different things — one about trust, one about navigation —
and both stay.

---

## The fourteen remedies

All 28 EN + FR strings are rewritten under these rules:

- **Never refer back to `doctor`.** Six current remedies say "run `josephine
  doctor`"; inside doctor that is circular. They all go.
- A **runnable command** where one exists, otherwise a concrete, verifiable
  gesture ("check airflow and dust").
- One line, imperative, fitting 80 columns including its indentation.
- `sudo` only where genuinely required, and visible when it is.
- The optional `hint` line serves diagnosis, not action: `systemctl status
  <unit>`.

Three checks gain a **contextual** remedy derived from the `CheckResult`; the
other eleven use their static text.

| Check | Contextual remedy | Source |
|---|---|---|
| `systemd` | one entry per failed unit — `sudo systemctl restart <unit>`, hint `systemctl status <unit>` | `top_processes` (`checks/systemd.rs:105`) |
| `cpu` | quotes the current top consumer verbatim | `top_processes[0]` |
| `memory` | quotes the current top consumer verbatim | `top_processes[0]` |

`cpu` and `memory` are free: `top_processes[0]` already reads
`firefox (PID 1234) — 87.3 %`, injected as-is with no parsing.

**Why `disk` is not contextual.** Naming the tight partition would need either a
new field on `CheckResult` — **35 construction sites across 15 files** — or an
unsafe demangle: `disk` stores its fullest-partition sentence *localised* in
`top_processes[0]`, and its per-partition metric names mangle the mount
(`/mnt/my_disk` → `usage_percent__mnt_my_disk`, which cannot be reversed
unambiguously). The disk check block prints `/home (ext4) : 91,2 % utilisé` two
lines above the section, so the information is not lost. The disk remedy stays
static: `josephine clean`.

---

## Removing `fix`

| File | Change |
|---|---|
| `crates/josephine/src/commands/fix_cmd.rs` | deleted, including its two `parse_failed_units` tests — core already tests its own parsing in `checks/systemd.rs` |
| `crates/josephine/src/commands/mod.rs:6` | drop `pub mod fix_cmd;` |
| `crates/josephine/src/cli.rs:9, 61-62, 123-125, 178` | import, `Commands::Fix` variant, dispatch arm, French `mut_subcommand("fix")` |
| `crates/josephine-core/src/voice.rs:146-200, 360-362` | drop `fix_tagline` / `fix_all_good`, rename `fix_hands_off` → `remedy_hands_off`, update `every_pool_is_bilingual_and_calm` |
| `crates/josephine/src/commands/explain_cmd.rs:87-88` | the systemd remedy literally advertises `josephine fix`; rewritten as part of the remedy pass |
| `README.md:119` | drop the command line from the usage block |
| `docs/CURRENT_STATE.md` | five mentions: l. 48, 74, 86, 91, 163 |
| `CHANGELOG.md` | `Removed` and `Added` entries under `[0.11.0]` |

Shell completions are generated from the clap tree and correct themselves.

`docs/ROADMAP.md:61` is **left untouched**: it is a historical table recording
what shipped in v0.4, and rewriting it would falsify the record.

### Explicit non-goal

`doctor --json` output is **unchanged**. Remedies do not join `CheckResult`, so
machine-readable output does not expose them. This is a real limitation, stated
rather than hidden: JSON serves supervision, not the human reader, and keeping
it out holds this increment to one clean change. Adding a `remedies` array to
the JSON contract is a candidate follow-up.

---

## Site

One occurrence, and it is the worst-placed one possible —
`site/templates/index.html:154`, inside the demo critical notification:

> *nginx.service s'est arrêté. En échec depuis 14:02. La remédiation guidée peut
> le relancer : `josephine fix`*

That sentence promises exactly what the product does not do. It becomes a
pointer to `josephine doctor`, with copy saying what actually happens: Joséphine
shows the restart command.

The CLI output showcase also gains the new "what's left to do" section, since
that is the visible novelty of the release.

**Out of scope:** the two images added to `site/static/` (`hero.png`, `why.png`)
are a separate decision, deliberately not folded into this spec. Three issues
are on record for that discussion: `why.png` renders garbled AI text
("TEMPERMIVEE", "Teintsservice", "PLAIES", "Since IANY") in its mock dashboard;
the pair weighs 4.6 MB on a page currently measured in tens of KB; and a chibi
mascot is a change of visual register for a site whose identity is the sober CLI
output on a night-watch ground, with an opaque grey gradient behind `why.png`
that will not sit on that ground.

---

## Tests

In core, around `remedy.rs`:

- systemd with two failed units → two remedies, correct unit names, each with
  its `hint`
- systemd with eight failed units → five entries plus a counted overflow line
- disk in `Critique` → falls back to its static `josephine clean` remedy, with
  no partition name and no `hint`
- a check in default with no contextual rule → falls back to static text,
  non-empty
- an `Info` result → no remedy at all (this is what guarantees a healthy machine
  prints no section)
- **a test iterating all fourteen checks**, requiring an `advice` entry for each
  with non-empty EN and FR — the guard rail that stops a fifteenth check from
  shipping without a remedy

Plus the standard quality gate (`cargo fmt --check`, `cargo clippy --workspace
--all-targets -- -D warnings`, `cargo test --workspace`) and a real-machine
`cargo run -p josephine -- doctor` by eye, in both languages.

---

## Release

**v0.11.0.** Removing a command is breaking, and in 0.x that is paid for with a
minor bump.
