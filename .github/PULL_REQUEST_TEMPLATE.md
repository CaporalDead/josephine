## Summary

<!-- What does this change and why? -->

## Type of change

- [ ] Bug fix
- [ ] New feature
- [ ] Documentation
- [ ] Refactor / chore

## Checklist

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] `cargo audit` and `cargo deny check` pass (`cargo install cargo-audit cargo-deny`)
- [ ] `nix flake check` passes (only if `flake.nix` or `packaging/nix/` changed)
- [ ] `CHANGELOG.md` updated under `[Unreleased]` (for user-visible changes)
- [ ] Commits follow Conventional Commits
- [ ] New user-facing strings ship in English and French via `i18n::t(en, fr)`; docs/identifiers in English
