# Joséphine — the words follow the face (increment 2: the copy)

**Date:** 2026-07-27
**Status:** Design approved, spec under review
**Scope:** Every user-facing string that still calls Joséphine a guardian angel
**Branch:** `feat/identity-words`
**Target release:** v0.12.0

---

## Context

Increment 1 gave Joséphine a face: four illustrations of a fox-spirit replaced a hand-drawn SVG of a halo and wings. It shipped deliberately alone, leaving the words untouched, so the live site currently shows a fox spirit beside copy that calls her a guardian angel. This increment closes that gap.

The name stays. *Joséphine* comes from the French series *Joséphine, ange gardien*, and the homage is not being erased — it is being loosened from the literal angel to the thing the series is actually about: someone who turns up, sorts out one household, and leaves.

### What the survey found

The repositioning is far more surgical than "rewrite the identity" suggests, because **the site almost never says "angel".** It says *guardian*, *watched over*, *keeping watch* — all of which describe a fox spirit as accurately as an angel. The eyebrow "Local Linux guardian", the tagline "Your machine, watched over — quietly.", the lede about "keeping watch between glances" all survive untouched.

The word "angel" appears on eleven live surfaces, two of which are published outside the repository and matter most: the **crates.io description** and the site's meta description, which feeds every share card.

### Decisions locked during brainstorming

- **The formula:** "Your computer's guardian spirit" / "L'esprit gardien de votre ordinateur". One word changes, and every existing "guardian"/"gardien" stays true.
- **The tone rule** in `CLAUDE.md` describes the register first and names the character second, so it survives the next change of mascot.
- **Social previews** are composed programmatically rather than generated, so their text is rendered by a font engine and cannot be garbled.
- **One release, v0.12.0**, carrying a single changelog entry for the identity change as a whole — increment 1 is merged but unreleased, so both halves ship together.

---

## The formula

| | |
|---|---|
| English | **Your computer's guardian spirit** |
| French | **L'esprit gardien de votre ordinateur** |

The README's variant keeps its adjective: "Your computer's quiet guardian spirit."

### Where it lands

| Surface | File |
|---|---|
| crates.io package description | `crates/josephine/Cargo.toml:10` |
| workspace description | `Cargo.toml:13` |
| `--help`, English | `crates/josephine/src/cli.rs:12,14` |
| `--help`, French | `crates/josephine/src/cli.rs:110` |
| the two tests pinning `--help` | `crates/josephine/tests/cli.rs:158,171` |
| site meta description, both languages | `site/config.toml:3,13` |
| README heading and image alt | `README.md:2,7` |
| README tone note | `README.md:24` |
| French README heading and tone row | `docs/README.fr.md:3,16` |
| contributor-facing tone note | `CONVENTIONS.md:29` |
| contributor greeting | `CONTRIBUTING.md:3` |
| product rule | `CLAUDE.md:17` |
| state-of-the-repo notes | `docs/CURRENT_STATE.md:80,114,201` |
| voice module doc comment | `crates/josephine-core/src/voice.rs:4` |

**One inconsistency is corrected in passing:** the workspace `Cargo.toml` carries a French description while the binary crate's is English. Both become English, the product's default language. This is not scope creep — the line is being edited anyway, and leaving one of the two in French would be choosing to preserve an accident.

---

## The tone rule

`CLAUDE.md:17` currently reads "Warm *guardian-angel* tone in **both** languages". It becomes:

> Warm, protective, quietly playful — never alarmist, and never `ERROR`/`FATAL`/`PANIC` in user-facing text. The register is Joséphine herself: a guardian spirit bound to one machine, who speaks up only when it matters and shows you the command rather than running it.

The final clause is not decoration. It writes into the product rule what deleting the `fix` command cost the project to learn: she shows, she does not do. A future contributor — or a future agent — reading only this line should not be able to propose a command that acts on the user's behalf without noticing it contradicts the rule.

`CONVENTIONS.md:29` and `docs/README.fr.md:16` carry shorter echoes of the same rule and are aligned to it. So does `README.md:22-24`, whose note on the two languages ends "The warm guardian-angel tone is preserved in both" — it becomes "The same warm, protective voice in both."

---

## The site copy

### The callout

`site/content/_index.md:36` and `_index.fr.md:36` are wrong on three counts at once: the angel, the claim that she "sorts it out" — which the removal of `fix` disproved — and the finger-snap, which is Mimie Mathy's signature gesture attached to a command that no longer exists.

English:

> **Joséphine is a guardian spirit, not a dashboard.** She notices when your machine needs a hand, tells you plainly what it needs — and shows you the command rather than running it behind your back. For people who'd rather not have to keep an eye on it themselves.

French:

> **Joséphine est un esprit gardien, pas un tableau de bord.** Elle remarque quand votre machine a besoin d'un coup de main, vous dit clairement ce qu'il lui faut — et vous montre la commande plutôt que de l'exécuter dans votre dos. Pour celles et ceux qui préfèrent ne pas avoir à surveiller eux-mêmes.

The middle clause deliberately echoes `voice.rs`'s own closing line, *"Je montre, vous appuyez. Rien ne s'exécute dans votre dos."* The landing page and the terminal now make the same promise in the same words.

### The caption

`site/templates/index.html`'s `.how__cap` reads "Envoyée veiller sur une machine. La vôtre." / "Sent to watch over one machine. Yours." *Sent* implies a sender, which is angelic. It becomes:

> **Attachée à une machine. La vôtre.** / **Bound to one machine. Yours.**

A spirit bound to one place is the folklore the illustrations come from, and it keeps the line's shape and rhythm exactly.

### The "no dashboards" contradiction

Increment 1 deferred this: `josephine-veille` illustrates the section whose lede opens "No dashboards. No graphs. No monitoring UI to keep open." The illustration shows her reading precisely that.

The lede is rewritten so the illustration becomes its evidence rather than its contradiction: the dashboard in the picture is *hers*, read on your behalf, which is exactly why you need none of your own. The same pass drops "she was sent here", which carries the same absent sender as the caption.

`site/templates/index.html:186,188` currently read:

> Pas de tableaux de bord. Pas de graphes. Aucune interface à garder ouverte. On lui a confié une mission — votre machine — et une chaîne toute simple lui suffit : du matériel jusqu'à un mot que vous pouvez suivre.

> No dashboards. No graphs. No monitoring UI to keep open. She was sent here with one assignment — your machine — and a simple chain is all she needs: from the hardware to a word you can act on.

They become:

> Aucun tableau de bord à garder ouvert : c'est elle qui lit les cadrans, pas vous. Une machine, une chaîne toute simple — du matériel jusqu'à un mot que vous pouvez suivre.

> No dashboard for you to keep open — she reads the dials so you don't have to. One machine, one simple chain: from the hardware to a word you can act on.

The heading above them, "Useful interventions, nothing more" / "Des interventions utiles, rien de plus", is unchanged and still true.

---

## The social previews

`resources/social-preview-{en,fr}.png` are the most out-of-date artefacts in the project. They predate not only the fox but the 2026-07-09 sober redesign: line-art angel with wings and halo, a pink heart, rainbow-gradient wordmark, "Have a great day! ✦", and "Josephine" without its accent. This is the image GitHub shows whenever anyone shares the repository.

They are **composed programmatically** by a committed script, `resources/make-social-preview.sh`, so that the next change of formula is one command rather than a design job — and so the text is rendered by a font engine and cannot come out garbled, which is the lesson increment 1 paid for.

- **1200 × 630, PNG.** GitHub's repository social-preview setting rejects WebP.
- Night ground `#0b0e19` with the site's ambient violet glow.
- `josephine-portrait-721.webp` on the left, the character already cut out.
- Wordmark `✦ Joséphine` in Source Code Pro (verified present on this machine), the formula in the file's language, and `github.com/systm-d/josephine`.
- Nothing else: no feature boxes, no hearts, no gradient.

The script takes the language as an argument and writes both files, so regenerating is `./resources/make-social-preview.sh en && ./resources/make-social-preview.sh fr`.

---

## What is deliberately left alone

Historical records keep saying "ange gardien", because they record what was true when they were written:

- everything under `docs/superpowers/` — past specs and plans
- `CHANGELOG.md`'s existing entries
- `docs/ROADMAP.md`'s dated tables

Rewriting these would falsify the record of what was decided, and when. The verification grep must therefore exclude them explicitly rather than expect zero hits repository-wide.

`voice.rs`'s phrasings themselves need no change — the 2026-07-09 detoning already removed every angel reference from them. Only the module's doc comment mentions one.

---

## Release

**v0.12.0**, with a single `Changed` entry covering the identity as a whole — the illustrations from increment 1 and the words from this one. Increment 1 is merged but unreleased, so both halves reach users in the same version, and no entry has to describe an intermediate state that was already resolved before anyone could see it.

---

## Verification

- `cargo test --workspace` — the two updated integration tests are the real guard: they fail if `cli.rs` is missed in either language.
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`.
- `zola build` clean, and both language builds contain the new formula and none of the old.
- `josephine --help` walked in both languages, using an isolated `HOME` rather than the maintainer's real config.
- The exhaustive grep for `guardian angel` / `ange gardien`, excluding the three historical areas named above, returns nothing.
- The two composed previews checked: exactly 1200 × 630, PNG, and looked at — the text legible, the character not clipped, the accent on Joséphine present.
