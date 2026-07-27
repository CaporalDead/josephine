# Conventions

The written source of truth for the standards shared across this project (and the
other CLIs built from the same `rust-cli-template` mood). Governance files link here
instead of restating these rules.

## Language & Edition

- Rust **edition 2024**, MSRV **1.85** (pinned via `rust-toolchain.toml`).
- Formatting: `rustfmt` with `max_width = 100`, edition 2024 (`rustfmt.toml`).
  `cargo fmt --check` must pass.
- Lints: `unsafe_code = "forbid"`; clippy `all = { level = "warn", priority = -1 }`,
  inherited per crate via `[lints] workspace = true`.
  `cargo clippy --workspace --all-targets -- -D warnings` must pass.

## Project shape

- Workspace: `josephine-core` (pure logic library) + `josephine` (binary, thin CLI).
- Module discipline: business logic lives in `josephine-core`; argument parsing,
  dispatch, and output formatting live in the `josephine` binary (`cli.rs`,
  `commands/`, `output/`). The CLI dispatch is thin — no business logic in `cli.rs`.
- **Linux-only** by design: checks rely on systemd (`systemctl`),
  `/sys/class/thermal`, and desktop notifications via libnotify.

## Language of text

- Documentation (README, this file, governance, the site) is in **English**.
- User-facing strings — CLI output and desktop notifications — ship in **English
  and French**, English by default. They are warm, protective and never
  alarmist; never `ERROR`/`FATAL`/`PANIC` in user-facing text.
- Code identifiers are in English.

## Git & releases

- **Conventional Commits** (`feat:`, `fix:`, `docs:`, `refactor:`, `chore:`,
  `test:`, `build:`, `style:`, `ci:`).
- **Keep a Changelog** format in `CHANGELOG.md`; **Semantic Versioning**.
- Dual license: **MIT OR Apache-2.0** (`LICENSE-MIT`, `LICENSE-APACHE`).

## Database

- SQLite via `rusqlite`. Schema changes are versioned migrations in
  `crates/josephine-core/migrations/` (`V00N__description.sql`), applied through the
  `schema_version` table. Multi-statement migrations must be wrapped in a transaction.

## Quality gate (run before every PR)

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
