# The Words Follow the Face — Implementation Plan (increment 2)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace "guardian angel" with "guardian spirit" on every live surface, retire the dead finger-snap, and recompose the two social previews so the words match the face increment 1 gave Joséphine.

**Architecture:** Five tasks, ordered so the riskiest lands first. Task 1 changes the CLI and its published metadata behind two integration tests that fail if either language is missed. Task 2 does the site copy, Task 3 the documentation and the product rule, Task 4 composes the social previews from a committed script, and Task 5 closes the changelog at v0.12.0 and verifies the lot.

**Tech Stack:** Rust workspace (`josephine-core` + `josephine`), `assert_cmd` integration tests, Zola static site with Tera templates, ImageMagick 7 (`magick`) with Source Code Pro for image composition.

**Spec:** [`docs/superpowers/specs/2026-07-27-josephine-identity-words-design.md`](../specs/2026-07-27-josephine-identity-words-design.md)

## Global Constraints

- **The formula is exactly:** `Your computer's guardian spirit` / `L'esprit gardien de votre ordinateur`. The README's variant keeps its adjective: `Your computer's quiet guardian spirit.`
- **Every user-facing string ships English AND French** — `i18n::t(en, fr)` in Rust, `{% if fr %}…{% else %}…{% endif %}` in `site/templates/index.html` (which sets `fr` at line 5), `{% if lang == 'fr' %}…{% else %}…{% endif %}` in `site/templates/base.html` (which does not).
- **Never `ERROR` / `FATAL` / `PANIC` / `CRASH` / `ÉCHEC`** in user-facing text.
- **Historical records keep saying "ange gardien"** and must NOT be edited: everything under `docs/superpowers/`, `CHANGELOG.md`'s existing entries, and `docs/ROADMAP.md`'s dated tables. They record what was true when written; rewriting them falsifies the record.
- Quality gate before every commit: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- The version becomes **0.12.0** in Task 5 and nowhere else.

## File Structure

| File | Responsibility |
|---|---|
| `crates/josephine/tests/cli.rs:158,171` | **Modified.** The two assertions that pin `--help` in each language |
| `crates/josephine/src/cli.rs:12,14,110` | **Modified.** The `about` line, English and French |
| `Cargo.toml:13`, `crates/josephine/Cargo.toml:10` | **Modified.** Workspace and crates.io descriptions |
| `site/config.toml:3,13` | **Modified.** Site meta descriptions, both languages |
| `site/content/_index.md:36`, `_index.fr.md:36` | **Modified.** The callout |
| `site/templates/index.html:186,188,199` | **Modified.** The "no dashboards" lede and the figure caption |
| `CLAUDE.md:17`, `CONVENTIONS.md:29`, `CONTRIBUTING.md:3` | **Modified.** The product rule and its echoes |
| `README.md:2,7,24`, `docs/README.fr.md:3,16` | **Modified.** Both READMEs |
| `docs/CURRENT_STATE.md:80,114,201` | **Modified.** State-of-the-repo notes |
| `crates/josephine-core/src/voice.rs:4` | **Modified.** Module doc comment |
| `resources/make-social-preview.sh` | **Created.** Composes both previews |
| `resources/social-preview-{en,fr}.png` | **Replaced.** |
| `CHANGELOG.md` | **Modified.** One entry for the identity change |

---

### Task 1: The formula in the CLI and its published metadata

The only task with a real test cycle. The two integration tests assert the `--help` line in each language; change them first, watch them fail, then change the code they pin.

**Files:**
- Modify: `crates/josephine/tests/cli.rs:158,171`
- Modify: `crates/josephine/src/cli.rs:12,14,110`
- Modify: `Cargo.toml:13`, `crates/josephine/Cargo.toml:10`

**Interfaces:**
- Consumes: nothing.
- Produces: the formula's exact wording, which Tasks 2-4 reuse verbatim.

- [ ] **Step 1: Change the two assertions**

In `crates/josephine/tests/cli.rs`, inside `help_about_follows_the_configured_language`, change line 158 from:

```rust
        .stdout(contains("guardian angel"));
```

to:

```rust
        .stdout(contains("guardian spirit"));
```

and line 171 from:

```rust
        .stdout(contains("ange gardien"));
```

to:

```rust
        .stdout(contains("esprit gardien"));
```

Change nothing else in that test — its isolated-`XDG_CONFIG_HOME` scaffolding is what makes the French branch testable at all.

- [ ] **Step 2: Run the test to verify it fails**

Run: `cargo test -p josephine --test cli help_about_follows_the_configured_language`
Expected: FAIL, on the English assertion first — the binary still prints "Your computer's guardian angel".

- [ ] **Step 3: Change the `about` line, both languages**

In `crates/josephine/src/cli.rs`, line 12 (the doc comment above `struct Cli`) and line 14 (the `#[command(...)]` attribute):

```rust
/// Your computer's guardian spirit
#[derive(Parser)]
#[command(name = "josephine", about = "Your computer's guardian spirit", version)]
```

And in `localize_help_fr`, line 110:

```rust
        .about("L'esprit gardien de votre ordinateur")
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cargo test -p josephine --test cli help_about_follows_the_configured_language`
Expected: PASS. Both branches now match.

- [ ] **Step 5: Change the two package descriptions**

`crates/josephine/Cargo.toml:10` — this is the **crates.io** description, published outside the repository:

```toml
description = "Joséphine — your computer's guardian spirit"
```

`Cargo.toml:13` — the workspace description. It is currently in French while the crate's is in English; both become English, the product's default language:

```toml
description = "Your computer's guardian spirit"
```

- [ ] **Step 6: Run the gate and commit**

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git add crates/josephine/tests/cli.rs crates/josephine/src/cli.rs Cargo.toml crates/josephine/Cargo.toml
git commit -m "feat(cli)!: Joséphine is a guardian spirit, not a guardian angel

The --help line and both package descriptions follow the face increment 1
gave her. The workspace description was in French while the crate's was in
English; both are English now, the product's default language."
```

---

### Task 2: The site copy

**Files:**
- Modify: `site/config.toml:3,13`
- Modify: `site/content/_index.md:36`, `site/content/_index.fr.md:36`
- Modify: `site/templates/index.html:186,188,199`

**Interfaces:**
- Consumes: the formula from Task 1.
- Produces: nothing.

- [ ] **Step 1: The meta descriptions**

`site/config.toml`, line 3 and line 13. These feed the `<meta name="description">` tag and every share card:

```toml
description = "Your computer's guardian spirit"
```

```toml
description = "L'esprit gardien de votre ordinateur"
```

- [ ] **Step 2: The callout**

`site/content/_index.md:36` currently reads:

```html
<p class="callout"><strong>Joséphine is a guardian angel, not a dashboard.</strong> She turns up when your machine needs a hand, sorts it out — about as close to a finger-snap as a terminal gets — then quietly steps back. For people who'd rather their computer simply took care of itself.</p>
```

Replace it with:

```html
<p class="callout"><strong>Joséphine is a guardian spirit, not a dashboard.</strong> She notices when your machine needs a hand, tells you plainly what it needs — and shows you the command rather than running it behind your back. For people who'd rather not have to keep an eye on it themselves.</p>
```

`site/content/_index.fr.md:36` currently reads:

```html
<p class="callout"><strong>Joséphine est un ange gardien, pas un tableau de bord.</strong> Elle arrive quand votre machine a besoin d'un coup de main, règle l'affaire — ce qu'un terminal fait de plus proche d'un claquement de doigts — puis s'efface discrètement. Pour celles et ceux qui préfèrent que leur ordinateur s'occupe simplement de lui-même.</p>
```

Replace it with:

```html
<p class="callout"><strong>Joséphine est un esprit gardien, pas un tableau de bord.</strong> Elle remarque quand votre machine a besoin d'un coup de main, vous dit clairement ce qu'il lui faut — et vous montre la commande plutôt que de l'exécuter dans votre dos. Pour celles et ceux qui préfèrent ne pas avoir à surveiller eux-mêmes.</p>
```

Three things changed, not one: the angel, the claim that she "sorts it out" (deleting the `fix` command in v0.11.0 disproved it), and the finger-snap — Mimie Mathy's signature gesture, which was attached to that dead command.

- [ ] **Step 3: The "no dashboards" lede**

`site/templates/index.html`, lines 186 and 188. The section is illustrated with `josephine-veille`, which shows her reading a dashboard — increment 1 deferred the contradiction to this one. The rewrite makes the illustration the lede's evidence instead.

Line 186 (French) becomes:

```
    Aucun tableau de bord à garder ouvert : c'est elle qui lit les cadrans, pas vous. Une machine, une chaîne toute simple — du matériel jusqu'à un mot que vous pouvez suivre.
```

Line 188 (English) becomes:

```
    No dashboard for you to keep open — she reads the dials so you don't have to. One machine, one simple chain: from the hardware to a word you can act on.
```

This also drops "She was sent here with one assignment" / "On lui a confié une mission", which carries an absent sender — the same angelic implication the caption has.

Leave the heading above them untouched: "Useful interventions, nothing more" / "Des interventions utiles, rien de plus" is still true.

- [ ] **Step 4: The figure caption**

`site/templates/index.html:199` currently reads:

```html
      <figcaption class="how__cap">{% if fr %}Envoyée veiller sur une machine. La vôtre.{% else %}Sent to watch over one machine. Yours.{% endif %}</figcaption>
```

Replace it with:

```html
      <figcaption class="how__cap">{% if fr %}Attachée à une machine. La vôtre.{% else %}Bound to one machine. Yours.{% endif %}</figcaption>
```

A spirit bound to one place is the folklore the illustrations come from, and the line keeps its shape and rhythm.

- [ ] **Step 5: Build and assert**

```bash
cd site && zola build && cd ..
grep -c 'guardian spirit' site/public/index.html
grep -c 'esprit gardien' site/public/fr/index.html
grep -ci 'guardian angel\|ange gardien\|finger-snap\|claquement' site/public/index.html site/public/fr/index.html || echo "no stale wording in either build (expected)"
```

Expected: `zola build` clean; the new formula present in each language's build; the stale wording absent from both, so the last grep exits non-zero and prints its reassurance line.

- [ ] **Step 6: Commit**

```bash
git add site/config.toml site/content/_index.md site/content/_index.fr.md site/templates/index.html
git commit -m "site: the copy follows the face

The callout was wrong on three counts — the angel, the claim she sorts
things out, and the finger-snap attached to a command deleted in v0.11.0.
The 'no dashboards' lede now treats its own illustration as evidence: the
dashboard in the picture is hers, read on your behalf."
```

---

### Task 3: The documentation and the product rule

**Files:**
- Modify: `CLAUDE.md:17`, `CONVENTIONS.md:28-29`, `CONTRIBUTING.md:3`
- Modify: `README.md:2,7,24`, `docs/README.fr.md:3,16`
- Modify: `docs/CURRENT_STATE.md:80,114,201`
- Modify: `crates/josephine-core/src/voice.rs:4`

**Interfaces:**
- Consumes: the formula from Task 1.
- Produces: nothing.

- [ ] **Step 1: The product rule**

`CLAUDE.md`, the bullet at lines 16-20, currently:

```markdown
- **English by default, French via `language: fr` in the config** (see
  `josephine-core/src/i18n.rs`). Warm *guardian-angel* tone in **both**
  languages; never `ERROR`/`FATAL`/`PANIC` in user-facing text. Every
  user-facing string must ship **English and French** — wrap literals in
  `i18n::t(en, fr)`, or use `match i18n::lang()` for interpolated ones.
```

becomes:

```markdown
- **English by default, French via `language: fr` in the config** (see
  `josephine-core/src/i18n.rs`). Warm, protective, quietly playful — never
  alarmist, and never `ERROR`/`FATAL`/`PANIC` in user-facing text. The
  register is Joséphine herself: a guardian spirit bound to one machine, who
  speaks up only when it matters and **shows you the command rather than
  running it**. Every user-facing string must ship **English and French** —
  wrap literals in `i18n::t(en, fr)`, or use `match i18n::lang()` for
  interpolated ones.
```

The final clause is load-bearing, not decoration: it writes into the product rule what deleting the `fix` command cost the project to learn.

- [ ] **Step 2: `CONVENTIONS.md`**

Lines 27-30 currently read:

```markdown
- Documentation (README, this file, governance, future site) is in **English**.
- User-facing strings — CLI output and desktop notifications — are in **French**
  and intentionally warm (Joséphine, the guardian angel). Never `ERROR`/`FATAL`/
  `PANIC` in user-facing text.
```

That second bullet carries a **second, worse error**: it says user-facing strings are in French, which stopped being true in v0.5.0 when English became the default and French an opt-in. Both errors are corrected:

```markdown
- Documentation (README, this file, governance, the site) is in **English**.
- User-facing strings — CLI output and desktop notifications — ship in **English
  and French**, English by default. They are warm, protective and never
  alarmist; never `ERROR`/`FATAL`/`PANIC` in user-facing text.
```

- [ ] **Step 3: `CONTRIBUTING.md:3`**

```markdown
Thanks for your interest in improving Joséphine — your computer's guardian spirit.
```

- [ ] **Step 4: `README.md`**

Line 2, the image alt:

```html
  <img src="resources/social-preview-en.png" alt="Joséphine — your computer's quiet guardian spirit" width="720">
```

Line 7, the tagline:

```html
<p align="center"><em>Your computer's quiet guardian spirit.</em></p>
```

Lines 22-24, the note on the two languages, currently ending "The warm guardian-angel tone is preserved in both":

```markdown
> Joséphine speaks **English by default**; set `language: fr` in
> `~/.config/josephine/config.yaml` for her French voice. The same warm,
> protective voice in both.
```

- [ ] **Step 5: `docs/README.fr.md`**

Line 3:

```markdown
> **L'esprit gardien de votre ordinateur.**
```

Line 16, the "Bienveillante" row:

```markdown
| **Bienveillante** | Jamais `ERROR`, `FATAL`, `PANIC`. Toujours un ton chaleureux et protecteur, jamais alarmiste. |
```

- [ ] **Step 6: `docs/CURRENT_STATE.md`**

Line 80, a section heading:

```markdown
### Voix & variété — l'esprit gardien (depuis 0.10.0)
```

Line 114, inside a parenthetical:

```markdown
  direct, calme, rassurant, jamais alarmiste (identité esprit gardien
```

Line 201, a table row:

```markdown
| Ton | Bilingue (anglais par défaut, français en option) ; identité esprit gardien, sucre visuel retiré — « chaleur sobre » depuis 0.7.0 ; **variété + caractère joueur** via `voice.rs` depuis 0.10.0 (lignes de caractère uniquement, faits d'alerte stables) |
```

- [ ] **Step 7: `crates/josephine-core/src/voice.rs:4`**

The module doc comment, not user-facing but read by everyone who edits the voice:

```rust
//! A guardian spirit with some character: warm, quietly playful, and never the
```

- [ ] **Step 8: Verify and commit**

```bash
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
grep -rn "guardian angel\|ange gardien" CLAUDE.md CONVENTIONS.md CONTRIBUTING.md README.md docs/README.fr.md docs/CURRENT_STATE.md crates/ || echo "clean (expected)"
```

Expected: gate green; the grep finds nothing and prints its reassurance line.

```bash
git add CLAUDE.md CONVENTIONS.md CONTRIBUTING.md README.md docs/README.fr.md docs/CURRENT_STATE.md crates/josephine-core/src/voice.rs
git commit -m "docs: the guardian is a spirit, and the tone rule says what it means

CLAUDE.md's rule now describes the register before naming the character,
and states outright that she shows the command rather than running it —
what deleting the fix command cost us to learn. CONVENTIONS.md also
claimed user-facing strings were French; that stopped being true in
v0.5.0."
```

---

### Task 4: Compose the social previews

The current `resources/social-preview-{en,fr}.png` predate not only the fox but the 2026-07-09 sober redesign: a line-art angel with wings and halo, a pink heart, a rainbow-gradient wordmark, "Have a great day! ✦", and "Josephine" without its accent. This is the image GitHub shows whenever anyone shares the repository.

**Files:**
- Create: `resources/make-social-preview.sh`
- Replace: `resources/social-preview-en.png`, `resources/social-preview-fr.png`

**Interfaces:**
- Consumes: the formula from Task 1; `site/static/josephine-portrait-721.webp`, committed by increment 1.
- Produces: nothing.

- [ ] **Step 1: Confirm the inputs exist**

```bash
test -f site/static/josephine-portrait-721.webp && identify -format "%wx%h %[channels]\n" site/static/josephine-portrait-721.webp
fc-list : family | tr ',' '\n' | grep -x "Source Code Pro" | head -1
magick -list format | grep -qi webp && echo "webp delegate present"
```

Expected: `721x824 srgba`, the font found, and the WebP delegate present. If Source Code Pro is missing, **stop and report** — substituting a different font silently changes the wordmark's character.

**A dependency to check first.** `josephine-portrait-721.webp` is the *cropped* portrait, delivered by PR #40. If the file is absent and `josephine-portrait-768.webp` is present instead, this branch was cut before that PR merged: **stop and report** rather than falling back to the uncropped square, which would put a differently-framed character on the share card than the one on the site.

- [ ] **Step 2: Write the script**

Create `resources/make-social-preview.sh`. **This script was run and its output inspected before this plan was written** — two non-obvious traps are already handled in it, and both are commented so nobody removes the fix as noise:

1. **`-compose` persists.** Setting `-compose Screen` for the glow layer leaves it set for the *next* `-composite` too, which silently screens the portrait onto the background and renders the character washed-out and translucent. The explicit `-compose Over` before the portrait is what prevents that.
2. **Source Code Pro has no `✦`.** U+2726 renders as nothing at all — verified, the canvas comes back empty. Symbola and FreeSerif do have it, but depending on either makes the script non-reproducible on another machine or in CI. The star is drawn from an SVG path instead.

```bash
#!/usr/bin/env bash
# Compose Joséphine's GitHub social preview — 1200x630 PNG, one per language.
#
# The text is rendered by the font engine rather than generated, so it cannot
# come out garbled. Regenerate after any change to the formula:
#   ./resources/make-social-preview.sh en && ./resources/make-social-preview.sh fr
set -euo pipefail

lang="${1:?usage: make-social-preview.sh <en|fr>}"
case "$lang" in
  en) formula="Your computer's guardian spirit" ;;
  fr) formula="L'esprit gardien de votre ordinateur" ;;
  *)  echo "unknown language: $lang (expected en or fr)" >&2; exit 1 ;;
esac

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
portrait="$root/site/static/josephine-portrait-721.webp"
out="$root/resources/social-preview-$lang.png"

night="#0b0e19"
star_col="#e9eaf6"
violet="#9a86e0"
dim="#8b91b4"
mono="Source-Code-Pro"

# The ✦ is drawn from a path, not typed: Source Code Pro has no glyph for
# U+2726, and the fonts that do (Symbola, FreeSerif) are not dependable on
# another machine. A path renders identically everywhere.
star_svg=$(mktemp --suffix=.svg)
trap 'rm -f "$star_svg"' EXIT
cat > "$star_svg" <<SVG
<svg xmlns="http://www.w3.org/2000/svg" width="34" height="34" viewBox="0 0 24 24">
  <path d="M12 0 L14.4 9.6 L24 12 L14.4 14.4 L12 24 L9.6 14.4 L0 12 L9.6 9.6 Z" fill="$violet"/>
</svg>
SVG

# Night ground, then a soft violet glow, then the portrait, then the text.
# The explicit `-compose Over` after the glow is load-bearing: without it the
# Screen operator carries over and washes the character out.
magick -size 1200x630 "xc:$night" \
  \( -size 1200x630 "xc:$night" \
     -fill "$violet" -draw 'circle 330,315 330,150' -blur 0x70 \) \
  -compose Screen -composite \
  -compose Over \
  \( "$portrait" -resize x520 \) -gravity west -geometry +60+0 -composite \
  "$star_svg" -gravity northwest -geometry +566+172 -composite \
  -font "$mono" -fill "$star_col" -pointsize 76 \
  -gravity northwest -annotate +614+140 "Joséphine" \
  -font "$mono" -fill "$dim" -pointsize 23 \
  -gravity northwest -annotate +618+258 "$formula" \
  -font "$mono" -fill "$violet" -pointsize 19 \
  -gravity southwest -annotate +620+92 "github.com/systm-d/josephine" \
  "$out"

echo "wrote $out ($(identify -format '%wx%h' "$out"))"
```

Make it executable: `chmod +x resources/make-social-preview.sh`

- [ ] **Step 3: Run it for both languages**

```bash
./resources/make-social-preview.sh en
./resources/make-social-preview.sh fr
identify -format "%f %wx%h %m\n" resources/social-preview-en.png resources/social-preview-fr.png
```

Expected: both report `1200x630 PNG`. GitHub's repository social-preview setting rejects WebP, so PNG is required, not preferred.

- [ ] **Step 4: Look at both images**

Open each and check, naming what you see rather than asserting it is fine:

- the accent on **Joséphine** is present and correctly rendered;
- the formula reads correctly and is not clipped at the right edge — the French string is 14 characters longer than the English and is the one at risk;
- the character is not cut off at the bottom or overlapped by text;
- nothing from the old preview survives: no wings, no halo, no heart, no gradient.

If the French formula overflows, reduce its `-pointsize` in the script rather than shortening the formula — the formula is fixed by the spec.

- [ ] **Step 5: Commit**

```bash
git add resources/make-social-preview.sh resources/social-preview-en.png resources/social-preview-fr.png
git commit -m "resources: recompose the social previews around the fox

The old ones predate both the fox and the 2026-07-09 sober redesign —
line-art angel, wings, halo, a heart, rainbow gradient, and Josephine
without her accent. Composed by script so the text is font-rendered
rather than generated, and so the next change of formula is one command."
```

---

### Task 5: Changelog, version, and verification

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `Cargo.toml` (workspace version)

**Interfaces:**
- Consumes: everything above.
- Produces: nothing.

- [ ] **Step 1: Write the changelog entry**

**Read `## [Unreleased]` before writing.** PR #39 (NixOS support) merges ahead of this branch and lands its own `### Added` and `### Fixed` subsections there — Nix packaging, and a fix for the `filesystem` check false-alarming on NixOS's read-only `/nix/store`. Those entries stay: v0.12.0 carries them alongside the identity change. Add the `### Changed` block below without disturbing them, and keep the conventional order Added → Changed → Fixed if it can be done without rewriting their text.

Under `## [Unreleased]`, add a single entry covering the identity change as a whole — increment 1 is merged but unreleased, so both halves reach users in this version:

```markdown
### Changed

- **Joséphine has a face, and the words to match.** She is a *guardian
  spirit* now, not a guardian angel: a small fox-spirit who watches one
  machine. Four illustrations replace the hand-drawn SVG on the site — a
  portrait beside the hero, a scene where she reads your dials, one above
  the fourteen checks, and one of her asleep on the tower when you reach the
  bottom of the page. The wording follows everywhere it appeared, including
  the `--help` line and the crates.io description.
- **The callout no longer promises what she doesn't do.** It claimed she
  "sorts it out" with something close to a finger-snap; removing `josephine
  fix` in v0.11.0 disproved the first and orphaned the second. She notices,
  she tells you plainly, and she shows you the command rather than running
  it — which is now written into the project's own tone rule.
```

- [ ] **Step 2: Close the section at 0.12.0**

Replace:

```markdown
## [Unreleased]
```

with:

```markdown
## [Unreleased]

## [0.12.0] - 2026-07-27
```

keeping the entries from Step 1 under `0.12.0`.

- [ ] **Step 3: Bump the version**

`Cargo.toml`, the `[workspace.package]` version, to `0.12.0`. Then check whether any member manifest pins it:

```bash
grep -rn '0\.11\.0' Cargo.toml crates/*/Cargo.toml packaging/
```

Any pinned `josephine-core` dependency version in `crates/josephine/Cargo.toml` must move in lockstep — Cargo's caret rule treats the minor digit as significant below 1.0, so `^0.11.0` does **not** match `0.12.0` and the workspace will fail to resolve. Then refresh the lockfile:

```bash
cargo build --workspace
```

- [ ] **Step 4: Run the full gate**

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd site && zola build && cd ..
```

Expected: all green, and the site builds clean.

- [ ] **Step 5: The exhaustive grep**

```bash
git grep -niE "guardian.?angel|ange gardien" -- \
  ':!docs/superpowers/' ':!CHANGELOG.md' ':!docs/ROADMAP.md'
```

`git grep` is scoped to tracked files by construction (no `target/` exclusion
needed) and has no extension list to go blind on — an earlier version of this
step filtered by `--include="*.md" --include="*.toml" --include="*.html"
--include="*.rs"`, which silently skipped `.nix`, `.rb`, and extensionless
files like `PKGBUILD` and `josephine.service`, and passed clean while seven
such surfaces still said "guardian angel".

Expected: **no output.** Every remaining hit lives in a historical record that must not be edited — past specs and plans, the changelog's own history, and the roadmap's dated tables.

- [ ] **Step 6: Walk `--help` in both languages**

```bash
tmp=$(mktemp -d)
XDG_CONFIG_HOME="$tmp/en" cargo run -q -p josephine -- --help | head -3
mkdir -p "$tmp/fr/josephine" && printf 'language: fr\n' > "$tmp/fr/josephine/config.yaml"
XDG_CONFIG_HOME="$tmp/fr" cargo run -q -p josephine -- --help | head -3
rm -rf "$tmp"
```

Expected: "Your computer's guardian spirit" and "L'esprit gardien de votre ordinateur". Use the isolated `XDG_CONFIG_HOME` shown — this machine's real config sets `language: fr`, so running without it tests only one branch and looks like it tested both.

- [ ] **Step 7: Commit**

```bash
git add CHANGELOG.md Cargo.toml Cargo.lock crates/josephine/Cargo.toml
git commit -m "release: v0.12.0"
```

Do **not** tag and do **not** push — tagging is the maintainer's call. Note in your report that the release body should carry the identity change, since the CHANGELOG is not read by someone upgrading in place.

---

## Known follow-ups, deliberately out of scope

- The `.github/PULL_REQUEST_TEMPLATE.md` checklist still says "User-facing strings stay in French", the same error corrected in `CONVENTIONS.md` here. It is not part of the identity change and deserves its own small commit.
- `docs/README.fr.md`'s command table omits seven shipped commands, including `explain` — pre-existing, flagged during increment 1.
- An 8 px horizontal overflow at a 320 px viewport predates all of this work.
