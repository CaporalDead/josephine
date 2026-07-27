# Joséphine — project guide for AI agents & contributors

Local Linux system guardian. Rust workspace: `josephine-core` (pure logic) +
`josephine` (binary, thin CLI).

## Read first

1. [docs/CURRENT_STATE.md](docs/CURRENT_STATE.md) — what exists
2. [docs/ROADMAP.md](docs/ROADMAP.md) — priorities
3. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — how the code is organized
4. [CONVENTIONS.md](CONVENTIONS.md) — shared standards (edition, fmt, lints, commits)
5. [CONTRIBUTING.md](CONTRIBUTING.md) — workflow & quality gate

## Product rules

- **English by default, French via `language: fr` in the config** (see
  `josephine-core/src/i18n.rs`). Warm, protective, quietly playful — never
  alarmist, and never `ERROR`/`FATAL`/`PANIC` in user-facing text. The
  register is Joséphine herself: a guardian spirit bound to one machine, who
  speaks up only when it matters and **shows you the command rather than
  running it**. Every user-facing string must ship **English and French** —
  wrap literals in `i18n::t(en, fr)`, or use `match i18n::lang()` for
  interpolated ones.
- 100% local, no cloud.
- Linux-only (systemd, `/sys/class/thermal`, libnotify).

## Where to change what

| Need | File |
|------|------|
| New check | `crates/josephine-core/src/checks/` + `config.rs` + `messages.rs` + an `Advice` entry in `remedy.rs` (a test fails without one) |
| Notification text | `crates/josephine-core/src/messages.rs` (EN + FR) |
| What a check watches, why it matters, how to fix it | `crates/josephine-core/src/remedy.rs` — `Advice` entries (EN + FR), shared by `josephine explain` and `doctor`'s closing to-do section |
| Varied "voice" lines (greetings, sign-offs, recoveries) | `crates/josephine-core/src/voice.rs` — pools of EN/FR phrasings; **flavour only**, never the facts of an alert |
| Any user-facing string | wrap in `i18n::t(en, fr)` / `match i18n::lang()` |
| CLI output | `crates/josephine/src/output/` |
| CLI command | `crates/josephine/src/commands/` |
| DB schema | `crates/josephine-core/migrations/` (versioned, `schema_version`) |

## Quality gate

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p josephine -- status
```
