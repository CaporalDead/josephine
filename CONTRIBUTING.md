# Contributing to Joséphine

Thanks for your interest in improving Joséphine — your computer's guardian angel.

## Before you start

- Read the [conventions](CONVENTIONS.md): edition, formatting, lints, commit style.
- By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).
- Joséphine is **Linux-only** (it relies on systemd, `/sys/class/thermal`, and
  libnotify). Changes should keep that target in mind.

## Where to start

- Issues labelled
  [good first issue](https://github.com/systm-d/josephine/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)
  are small and self-contained — a good way in.
- Issues labelled
  [help wanted](https://github.com/systm-d/josephine/issues?q=is%3Aissue+is%3Aopen+label%3A%22help+wanted%22)
  are ready for anyone to pick up.
- [docs/ROADMAP.md](docs/ROADMAP.md) explains where the project is going; open
  issues labelled `roadmap` are pre-scoped, delegable pieces of it.
- To find your way around the code, read
  [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) (structure, and how to add a new
  check) and [docs/CURRENT_STATE.md](docs/CURRENT_STATE.md) (what ships today).

For anything larger than a bug fix, open an issue first so we can agree on the
shape — it saves everyone a round-trip once the pull request is up.

## Development setup

```sh
git clone https://github.com/systm-d/josephine
cd josephine
cargo build
```

The toolchain is pinned by `rust-toolchain.toml` (stable + rustfmt + clippy).

## Language of text

Documentation, governance files, the website and code identifiers are in
**English**. User-facing strings — CLI output and desktop notifications — ship
in **both English and French**: English is the default, French applies when the
user sets `language: fr` in `~/.config/josephine/config.yaml`.

- Wrap every user-facing literal in `i18n::t(en, fr)`
  (`crates/josephine-core/src/i18n.rs`), or use `match i18n::lang()` when the
  string is interpolated. A string that ships in only one language is a bug.
- The tone, in both languages, is warm, protective, quietly playful and never
  alarmist — and never `ERROR` / `FATAL` / `PANIC` in user-facing text.

## Quality gate

Run these three every time, before opening a pull request. They are what CI's
`lint` and `test` jobs enforce:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs four more jobs — `security`, `bench-smoke`, `nix` and `coverage`. The
first three need extra tooling, and reproducing them locally is the difference
between a green gate and a red CI:

```sh
cargo audit            # security — cargo install cargo-audit
cargo deny check       # security — cargo install cargo-deny
cargo bench --no-run   # bench-smoke — benches must still compile
nix flake check        # nix — needs Nix; builds the flake and a NixOS VM test.
                       # Only worth running if you touched flake.nix
                       # or packaging/nix/
```

`coverage` (`cargo tarpaulin --workspace`, uploaded to Codecov) is
informational only: it never blocks a merge, so don't lose sleep over it.

One last detail: CI runs the test and bench jobs with `--locked`, on Ubuntu
22.04/24.04 and Fedora 40/41. `Cargo.lock` is committed, so any change that
would modify it turns CI red — commit the updated lockfile alongside it.

## Commits & pull requests

- Use [Conventional Commits](https://www.conventionalcommits.org/)
  (`feat:`, `fix:`, `docs:`, `refactor:`, `chore:`, `test:`, …).
- Add a `CHANGELOG.md` entry under `[Unreleased]` for user-visible changes.
- One focused change per PR. Fill in the pull request template.

## Reporting bugs & ideas

Open an issue using the bug or feature template. For security issues, do **not**
open a public issue — see [SECURITY.md](SECURITY.md).

## License of contributions

By submitting a contribution you agree that it is licensed under the same terms
as the project: **MIT OR Apache-2.0, at the user's option** — see
[LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE). There is no
CLA to sign and no DCO sign-off: opening the pull request is the agreement, and
no `Signed-off-by` trailer is required.
