# Contributing to Joséphine

Thanks for your interest in improving Joséphine — your computer's guardian spirit.

## Before you start

- Read the [conventions](CONVENTIONS.md): edition, formatting, lints, commit style.
- By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).
- Joséphine is **Linux-only** (it relies on systemd, `/sys/class/thermal`, and
  libnotify). Changes should keep that target in mind.

## Development setup

```sh
git clone https://github.com/systm-d/josephine
cd josephine
cargo build
```

The toolchain is pinned by `rust-toolchain.toml` (stable + rustfmt + clippy).

## Quality gate

Run this before opening a pull request — CI enforces the same:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Commits & pull requests

- Use [Conventional Commits](https://www.conventionalcommits.org/)
  (`feat:`, `fix:`, `docs:`, `refactor:`, `chore:`, `test:`, …).
- Add a `CHANGELOG.md` entry under `[Unreleased]` for user-visible changes.
- Keep user-facing strings (CLI output, notifications) in French and warm; keep
  docs and code identifiers in English.
- One focused change per PR. Fill in the pull request template.

## Reporting bugs & ideas

Open an issue using the bug or feature template. For security issues, do **not**
open a public issue — see [SECURITY.md](SECURITY.md).
