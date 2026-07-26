# Joséphine — a face for the guardian (increment 1: the visuals)

**Date:** 2026-07-25
**Status:** Design approved, spec under review
**Scope:** Illustrated identity — four generated images integrated into the site, the hand-drawn SVG guardian retired
**Branch:** `feat/visual-identity`

---

## Context

Joséphine has never had a face. The site's identity since the 2026-07-09 redesign is *"night watch"*: a dark cosmic ground, monospace-forward typography, a discreet `✦`, and **the real sober CLI output as the hero**. The only illustration is `m::guardian()` — a hand-written inline SVG showing a halo, discreet wings, and an open laptop with Joséphine present on the screen as a `✦`. It is captioned *"Envoyée veiller sur une machine. La vôtre."*

The owner has produced four illustrations of a character — a small white fox-spirit in a dark purple kimono, surrounded by violet soul-flames, with a `>_` prompt mark on her forehead — and wants to give Joséphine that face.

### The decision that governs everything else

This is **not** a decorative addition. The owner ruled for a **full repositioning**: Joséphine becomes a household spirit rather than a guardian angel. That contradicts the current identity in a specific, load-bearing way — the name comes from the French series *Joséphine, ange gardien*, and `CLAUDE.md` carries "warm *guardian-angel* tone" as a product rule.

The work is therefore split in two increments, because producing images is iterative and subjective while rewriting copy is mechanical once the character is settled:

- **Increment 1 (this spec):** the visuals. Assets, formats, placement, the SVG's retirement.
- **Increment 2 (later):** the words. Site copy, the CLI's `about` line and the two integration tests pinning it, both READMEs, `CLAUDE.md`'s product rule, and the social previews.

Increment 1 deliberately leaves every "ange gardien" string in place. The page will briefly show a fox spirit next to angel wording; that is a known, bounded intermediate state, resolved by increment 2 before either ships.

---

## The assets

Four images, all generated, all 1:1 or 3:2, all with a **real alpha channel** (verified: all four corners at `alpha=0`).

| File | Subject | Native |
|---|---|---|
| `hero.png` | She stands behind a desk — monitor, tower, plant, mug — arms open, glowing paws | 1536×1024 |
| `why.png` | She points at a monitor showing a legible dashboard, lantern in hand | 1536×1024 |
| `checks.png` | Clean full-body portrait, a soul-flame in each palm, no scenery | 1024×1024 |
| `footer.png` | Asleep on a PC tower, "Zzz…", two flames dozing with her | 1024×1024 |

### The rule these assets must obey

**No shipped image carries informative text.** This is a hard rule for this spec and beyond.

It was learned the expensive way. The first `why.png` rendered its mock dashboard as "TEMPERMIVEE", "Teintsservice", "PLAIES". The first `checks.png` labelled fourteen medallions with names that were both garbled *and* factually wrong — it advertised `BACKUP`, `PROCESSES` and `UPTIME`, which Joséphine does not have, while omitting memory, inodes, filesystem and timesync, which she does. A landing page that names checks the tool doesn't run is the same defect as the `josephine fix` command that promised a repair it never performed, on a more visible surface.

The page already renders the fourteen checks as real HTML: exact names, translated, readable by a screen reader, and automatically correct when a fifteenth check ships. **Illustrations carry mood; HTML carries data.**

Two admitted exceptions, both pictograms rather than information: the `>_` prompt on her forehead and lantern, and the "Zzz…" in `footer.png`. Neither can rot or become false.

### One recorded derogation

`why.png` — shipped as `josephine-veille` — **breaks this rule, knowingly.** Its mock dashboard renders "UPOATES" for UPDATES, "all Funning" for all running, "avalbbie" for available, and "15G / 310" for 18G / 31G. The handwritten sticky note below the screen is pure scribble.

The owner ruled on 2026-07-26 to ship it anyway. The reasoning: served at roughly 416 CSS pixels, those cells are small enough that no one reads them in passing — the defect surfaces only on deliberate zoom or a high-density display — and this is the third generation of the image to fail on exactly this, with no sign that another attempt would fix it.

It is recorded here rather than left silent so that a future reader finds a decision and not an oversight. The rule above still stands for every future asset; this is the exception that proves someone looked.

### Formats and sizes

Only **WebP** enters the repository. No PNG fallback — WebP has been supported by every targeted browser since Safari 14 (2020).

The PNG sources total roughly 8 MB and are generation artefacts; they stay out of git. If the owner wants them archived, `resources/` is the consistent place, but that is an explicit decision, not a default.

Two widths per image, served via `srcset`, each sized for its slot rather than uniformly:

| Slot | Image | Widths | Weight |
|---|---|---|---|
| Hero, right column | `checks` | 512 / 768 | 45 / 96 KB |
| "How she works" | `why` | 768 / 1024 | 103 / 173 KB |
| Checks grid | `hero` | 768 / 1024 | 110 / 175 KB |
| Footer | `footer` | 384 / 512 | 36 / 61 KB |

### Renaming

The filenames describe where the owner first imagined each image, not where it lands. They are renamed on integration, since nothing references them yet:

| From | To |
|---|---|
| `checks.png` | `josephine-portrait.webp` |
| `why.png` | `josephine-veille.webp` |
| `hero.png` | `josephine-bureau.webp` |
| `footer.png` | `josephine-repos.webp` |

---

## Placement

The layout was chosen from three mocked alternatives (option C: *"les deux, chacune à sa place"*, extended to four images once `checks.png` and `footer.png` arrived).

**Hero** becomes two columns: eyebrow, title, tagline, lede and CTA on the left; `josephine-portrait` on the right. It collapses to one column at **48rem**, the breakpoint `.how` already uses.

On narrow screens the portrait sits **under** the CTA. This is deliberately the opposite of `.how__art`, which carries `order: -1` and floats its illustration above its text — in the hero, the title and the install button must remain the first thing visible on a phone, so the portrait must not be given a negative order.

The real CLI readout (`.term`) stays exactly where it is, below the hero block: the terminal keeps its place in the identity, it simply no longer carries the page alone.

`josephine-portrait` earns the hero slot because it is the only square, scenery-free image of the four. A square sits in a text-adjacent column without cropping, and it compresses better than the landscape scenes — 96 KB against 175 KB for the same visual weight.

**"How she works"** — the `.how__art` figure. The `m::guardian()` SVG is **deleted** from `macros.html`, comment, halo and wings included, and its `.guardian` rule leaves `main.scss`. `josephine-veille` takes the slot: it is the explanatory image, showing her reading a dashboard, and its landscape framing suits the two-column grid beside the `.flow` nodes.

**Checks grid** — `josephine-bureau` sits **above** the fourteen check cards, full width and centred, as the section's opening image. Not beside them: the grid is already a dense multi-column block, and squeezing an illustration into it would either crush the image or force the cards into a column too narrow for their labels. She is surrounded by her machine, arms open: "here is what she watches."

**Footer** — `josephine-repos`. The page ends on her asleep on the tower, which is the emotional close the product's promise deserves.

### Markup requirements

Every `<img>` carries explicit `width` and `height` so the page does not reflow while images load. The hero portrait is `loading="eager"`; the other three are `loading="lazy"` — only the portrait is on the critical path.

Alt text is written in **English and French**, like every other string on the site. `josephine-veille`'s alt describes the scene without transcribing the mock dashboard's numbers: the real measurements live elsewhere on the page, in HTML.

### What is deliberately not touched

`base.html`'s `<div class="halo">` is an ambient background glow, not an angel's halo — it stays. The caption *"Envoyée veiller sur une machine. La vôtre."* stays too: it is a sentence, and sentences belong to increment 2.

---

## Page weight

The site currently weighs tens of KB. Four illustrations take it to roughly **505 KB** on a wide screen and **294 KB** on a phone. That is a tenfold increase, stated plainly so it is a decision rather than a discovery.

What makes it acceptable: only the hero portrait is on the critical path, at **96 KB**. The other three load as the visitor scrolls. First paint costs one image, not four.

---

## Derivatives

**The favicon stays `✦`.** A detailed chibi reduced to 16 pixels becomes violet mush, and the `✦` has the rare property of being the exact glyph the CLI prints in its own header — browser tab and terminal say the same thing. Putting the fox in the tab would need a drawing simplified *for that size*, which is separate work, not a downscale.

**Social previews are out of scope.** `resources/social-preview-{en,fr}.png` carry the wording "Your computer's guardian angel" / "L'ange gardien de votre ordinateur" — words, therefore increment 2. They must also stay PNG: GitHub's repository social-preview setting rejects WebP.

---

## Verification

- `zola build` clean, and the built pages spot-checked in both languages.
- Real page weight measured before and after, not estimated.
- Rendering checked at a narrow viewport: the portrait sits below the CTA, no horizontal overflow.
- All four corners of every shipped image confirmed at `alpha=0`:
  `magick <file> -format "%[fx:p{2,2}.a]" info:` and the three other corners.
- A final read confirming no shipped image carries informative text, the `>_` and "Zzz…" pictograms excepted.
- No `.rs` file changes: this increment touches only `site/` and the assets.

---

## Explicit non-goals

- Any copy change. Every "ange gardien" / "guardian angel" string survives increment 1 untouched.
- `CLAUDE.md`'s product rule.
- The favicon.
- The social previews.
- `docs/ROADMAP.md`, and the planning documents under `docs/superpowers/` — historical records.
