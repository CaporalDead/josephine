# Joséphine's Face — Implementation Plan (increment 1: the visuals)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Put four illustrations of Joséphine onto the site, in four distinct slots, and retire the hand-drawn SVG guardian they replace.

**Architecture:** Four generated PNGs become eight WebP files — two widths each, sized per slot — served with `srcset`. The hero grows a second column for the portrait; the `.how__art` figure swaps its inline SVG for the watching scene; the checks section gains an opening image; the footer gains a closing one. No Rust changes, and **no copy changes at all** — every "ange gardien" string survives untouched for increment 2.

**Tech Stack:** Zola static site generator, Tera templates, SCSS compiled by Zola, ImageMagick 7 (`magick`) with libwebp for asset production.

**Spec:** [`docs/superpowers/specs/2026-07-25-josephine-visual-identity-design.md`](../specs/2026-07-25-josephine-visual-identity-design.md)

## Global Constraints

- **No copy changes.** Not one user-facing sentence is edited in this increment. Alt text is *new* text and is written EN + FR; everything already on the page stays exactly as it is, including every "ange gardien" / "guardian angel" string and the caption "Envoyée veiller sur une machine. La vôtre."
- **Every new user-facing string ships English AND French**, using the templates' existing pattern: `{% if fr %}…{% else %}…{% endif %}` in `index.html`, and `{% if lang == 'fr' %}…{% else %}…{% endif %}` in `base.html` — note the two files differ, `index.html` defines `{% set fr = lang == 'fr' %}` at line 5 and `base.html` does not.
- **No shipped image carries informative text.** The `>_` prompt mark and the "Zzz…" bubble are admitted as pictograms. Nothing else.
- **Only WebP enters the repository.** The PNG sources stay out of git.
- Every `<img>` carries explicit `width` and `height` attributes, `loading="eager"` for the hero portrait and `loading="lazy"` for the other three, and `decoding="async"`.
- **Do not add `order: -1` to the hero art.** `.how__art` has it and floats its illustration above its text; the hero must do the opposite so the title and install button stay first on a phone.
- Breakpoint for the hero collapse is **48rem**, the one `.how` already uses (`site/sass/main.scss:606`).
- `docs/ROADMAP.md` and everything under `docs/superpowers/` are untouched.

## File Structure

| File | Responsibility |
|---|---|
| `site/static/josephine-portrait-{512,768}.webp` | **New.** Hero portrait, from `checks.png` |
| `site/static/josephine-veille-{768,1024}.webp` | **New.** Watching scene, from `why.png` |
| `site/static/josephine-bureau-{768,1024}.webp` | **New.** Desk scene, from `hero.png` |
| `site/static/josephine-repos-{384,512}.webp` | **New.** Sleeping, from `footer.png` |
| `site/templates/macros.html` | **Modified.** `guardian()` macro deleted |
| `site/templates/index.html` | **Modified.** Hero restructured; `.how__art` figure; checks-section image |
| `site/templates/base.html` | **Modified.** Footer image |
| `site/sass/main.scss` | **Modified.** `.hero` becomes a grid; `.guardian` removed; image rules added |

### A note on the source PNGs

The four PNGs are **untracked** and live only in the maintainer's main checkout at
`/home/kdelfour/Workspace/Professionel/systm-D/josephine/site/static/`:
`checks.png`, `why.png`, `hero.png`, `footer.png`.

They are not in this worktree and never enter git. Task 1 reads them from that absolute path. If they are missing, **stop and report** rather than substituting anything — the assets are the whole point of this increment.

---

### Task 1: Produce and verify the eight WebP assets

**Files:**
- Create: `site/static/josephine-portrait-512.webp`, `josephine-portrait-768.webp`, `josephine-veille-768.webp`, `josephine-veille-1024.webp`, `josephine-bureau-768.webp`, `josephine-bureau-1024.webp`, `josephine-repos-384.webp`, `josephine-repos-512.webp`

**Interfaces:**
- Consumes: the four untracked PNGs named above.
- Produces: the eight filenames above, referenced verbatim by Tasks 2–5.

- [ ] **Step 1: Confirm the sources exist and are cut out**

```bash
SRC=/home/kdelfour/Workspace/Professionel/systm-D/josephine/site/static
for f in checks why hero footer; do
  test -f "$SRC/$f.png" || { echo "MISSING: $f.png"; exit 1; }
  w=$(identify -format "%w" "$SRC/$f.png"); h=$(identify -format "%h" "$SRC/$f.png")
  printf "%-8s %sx%s corners:" "$f" "$w" "$h"
  for xy in "2,2" "$((w-3)),2" "2,$((h-3))" "$((w-3)),$((h-3))"; do
    printf " %s" "$(magick "$SRC/$f.png" -format "%[fx:p{$xy}.a]" info:)"
  done
  echo
done
```

Expected: four lines, every corner value `0`. A non-zero corner means the image has a painted background instead of transparency — stop and report it; it must be re-exported, not worked around.

- [ ] **Step 2: Generate the eight files**

```bash
SRC=/home/kdelfour/Workspace/Professionel/systm-D/josephine/site/static
gen () { magick "$SRC/$1.png" -resize "$2x" -define webp:method=6 -quality 82 "site/static/$3-$2.webp"; }
gen checks 512  josephine-portrait
gen checks 768  josephine-portrait
gen why    768  josephine-veille
gen why    1024 josephine-veille
gen hero   768  josephine-bureau
gen hero   1024 josephine-bureau
gen footer 384  josephine-repos
gen footer 512  josephine-repos
```

- [ ] **Step 3: Verify the output**

```bash
for f in site/static/josephine-*.webp; do
  printf "%-40s %6s KB  %s  %s\n" "$(basename $f)" \
    "$(( $(stat -c%s $f)/1024 ))" \
    "$(identify -format '%wx%h' $f)" \
    "$(identify -format '%[channels]' $f)"
done
```

Expected: eight files. Every one reports `srgba` — if any reports `srgb`, its alpha was dropped and the resize/convert must be redone. Sizes should land near 45, 96, 103, 173, 110, 175, 36 and 61 KB respectively; a wild deviation means the wrong source was used.

- [ ] **Step 4: Commit**

```bash
git add site/static/josephine-*.webp
git commit -m "assets: eight WebP renditions of Joséphine, two widths per slot"
```

---

### Task 2: Retire the SVG guardian, install the watching scene

The most self-contained visible change: one figure swaps its contents. Do this before the hero so the SVG is gone early and cannot be accidentally left in two places.

**Files:**
- Modify: `site/templates/macros.html:56-89` (delete the `guardian()` macro)
- Modify: `site/templates/index.html:183` (the `.how__art` figure)
- Modify: `site/sass/main.scss:435` (`.guardian` rule) and `:606-610` (the 48rem block)

**Interfaces:**
- Consumes: `josephine-veille-768.webp`, `josephine-veille-1024.webp` from Task 1.
- Produces: nothing later tasks depend on.

- [ ] **Step 1: Delete the macro**

In `site/templates/macros.html`, delete the whole block from the comment above `{% macro guardian() %}` through `{% endmacro guardian %}` — the comment ("Signature scene: Joséphine, guardian angel of your machine…"), the `<svg class="guardian">` element, and the macro wrapper. Leave the other macros in the file untouched.

- [ ] **Step 2: Replace the figure's contents**

`site/templates/index.html:183` currently reads:

```html
    <figure class="how__art">{{ m::guardian() }}<figcaption class="how__cap">{% if fr %}Envoyée veiller sur une machine. La vôtre.{% else %}Sent to watch over one machine. Yours.{% endif %}</figcaption></figure>
```

Replace it with:

```html
    <figure class="how__art">
      <img class="art" src="{{ get_url(path='josephine-veille-1024.webp') }}"
           srcset="{{ get_url(path='josephine-veille-768.webp') }} 768w, {{ get_url(path='josephine-veille-1024.webp') }} 1024w"
           sizes="(max-width: 48rem) 90vw, 26rem"
           width="1024" height="683" loading="lazy" decoding="async"
           alt="{% if fr %}Joséphine, petite gardienne renard, lit un tableau de bord sur l'écran d'un ordinateur, une lanterne à la main{% else %}Joséphine, a small fox guardian, reads a dashboard on a computer screen, lantern in hand{% endif %}">
      <figcaption class="how__cap">{% if fr %}Envoyée veiller sur une machine. La vôtre.{% else %}Sent to watch over one machine. Yours.{% endif %}</figcaption>
    </figure>
```

The caption is **unchanged** — it is copy, and copy belongs to increment 2. The alt text describes the scene without transcribing the dashboard's numbers.

- [ ] **Step 3: Swap the SCSS rule**

In `site/sass/main.scss`, replace the `.guardian` rule (line 435, `.guardian { width: 100%; max-width: 24rem; height: auto; }`) with:

```scss
.art { width: 100%; height: auto; display: block; }
.how__art .art { max-width: 26rem; }
```

Then in the `@media (max-width: 48rem)` block (line 606), replace `.guardian { max-width: 18rem; }` with:

```scss
  .how__art .art { max-width: 18rem; }
```

Leave `.how__art { order: -1; }` in that block exactly as it is — the how-section illustration is meant to float above its text on narrow screens.

- [ ] **Step 4: Build and assert**

```bash
cd site && zola build && cd ..
grep -c 'josephine-veille' site/public/index.html site/public/fr/index.html
grep -c 'class="guardian"' site/public/index.html site/public/fr/index.html || echo "guardian svg gone (expected)"
grep -c 'Envoyée veiller sur une machine' site/public/fr/index.html
```

Expected: `zola build` clean; `josephine-veille` appears in both language builds; `class="guardian"` appears **zero** times (grep exits non-zero, printing the reassurance line); the untouched caption still appears once in the French build.

- [ ] **Step 5: Commit**

```bash
git add site/templates/macros.html site/templates/index.html site/sass/main.scss
git commit -m "site: the watching scene replaces the hand-drawn SVG guardian"
```

---

### Task 3: The hero grows a second column for the portrait

The largest layout change. `.hero` is centred today (`text-align: center`, `margin: … auto` on the tagline and lede, `justify-content: center` on `.cta` and `.trust`); it becomes a two-column grid with the text left-aligned, reverting to the current centred presentation below 48rem.

**Files:**
- Modify: `site/templates/index.html:17-35` (wrap the text, add the figure)
- Modify: `site/sass/main.scss:79` (`.hero`), `:102-103` (tagline, lede), `:159` (`.hero .term`), `:611` (the 40rem block)

**Interfaces:**
- Consumes: `josephine-portrait-512.webp`, `josephine-portrait-768.webp` from Task 1; the `.art` class from Task 2.
- Produces: nothing later tasks depend on.

- [ ] **Step 1: Restructure the hero markup**

In `site/templates/index.html`, the hero block currently opens at line 17 with `<div class="wrap hero">` and holds six text elements followed by `<div class="term" …>`. Wrap the six text elements in a `.hero__text` div and insert the figure after it, leaving `.term` as a direct child of `.hero`:

```html
  <div class="wrap hero">
    <div class="hero__text">
      <p class="eyebrow">{{ section.extra.eyebrow }}</p>
      <h1><span class="star">✦</span> Joséphine</h1>
      <p class="tagline">{{ section.extra.tagline }}</p>
      <p class="lede">{{ section.extra.lede }}</p>
      <div class="cta">
        <a class="button button--primary" href="#install">{{ section.extra.cta2 }}</a>
        <a class="button" href="{{ config.extra.repo_url }}">{{ section.extra.cta }}</a>
      </div>
      <ul class="trust" aria-label="{% if fr %}Principes{% else %}Principles{% endif %}">
        <li>{% if fr %}Pas de cloud{% else %}No cloud{% endif %}</li>
        <li>{% if fr %}Pas de télémétrie{% else %}No telemetry{% endif %}</li>
        <li>{% if fr %}Pas de compte{% else %}No account{% endif %}</li>
        <li>{% if fr %}Écrite en Rust{% else %}Written in Rust{% endif %}</li>
        <li>MIT / Apache-2.0</li>
      </ul>
    </div>

    <figure class="hero__art">
      <img class="art" src="{{ get_url(path='josephine-portrait-768.webp') }}"
           srcset="{{ get_url(path='josephine-portrait-512.webp') }} 512w, {{ get_url(path='josephine-portrait-768.webp') }} 768w"
           sizes="(max-width: 48rem) 60vw, 22rem"
           width="768" height="768" loading="eager" decoding="async"
           alt="{% if fr %}Joséphine, petite gardienne renard en kimono sombre, une flamme violette dans chaque paume{% else %}Joséphine, a small fox guardian in a dark kimono, a violet flame in each palm{% endif %}">
    </figure>
```

The six text elements keep their exact contents — every string, including the `aria-label`, is copied over unchanged. `.term` and everything after it stay where they are.

- [ ] **Step 2: Make `.hero` a grid**

In `site/sass/main.scss`, replace line 79:

```scss
.hero { padding: 4.5rem 0 3.5rem; text-align: center; }
```

with:

```scss
.hero {
  padding: 4.5rem 0 3.5rem;
  display: grid;
  grid-template-columns: 1fr minmax(0, 22rem);
  gap: 2.5rem 3rem;
  align-items: center;
}
.hero__text { text-align: left; }
.hero__text .tagline, .hero__text .lede { margin-left: 0; margin-right: 0; }
.hero__text .cta, .hero__text .trust { justify-content: flex-start; }
.hero__art { margin: 0; }
```

Then make the terminal readout span both columns — replace line 159:

```scss
.hero .term { max-width: 44rem; margin: 2.6rem auto 0; }
```

with:

```scss
.hero .term { grid-column: 1 / -1; max-width: 44rem; margin: 2.6rem auto 0; }
```

- [ ] **Step 3: Restore the centred presentation below 48rem**

Add a `.hero` block to the existing `@media (max-width: 48rem)` rule in `site/sass/main.scss` (line 606), alongside the `.how` rules already there:

```scss
  .hero { grid-template-columns: 1fr; }
  .hero__text { text-align: center; }
  .hero__text .tagline, .hero__text .lede { margin-left: auto; margin-right: auto; }
  .hero__text .cta, .hero__text .trust { justify-content: center; }
  .hero__art { max-width: 16rem; margin: 0 auto; }
```

**Do not add an `order` property here.** The portrait must follow the text in the collapsed layout, and DOM order already delivers that. Adding `order: -1` — as `.how__art` does — would push the illustration above the title and the install button.

- [ ] **Step 4: Build and assert**

```bash
cd site && zola build && cd ..
grep -c 'josephine-portrait' site/public/index.html site/public/fr/index.html
grep -c 'hero__text' site/public/index.html
grep -o 'loading="[a-z]*"' site/public/index.html | sort | uniq -c
```

Expected: build clean; the portrait appears in both builds; `hero__text` appears once; exactly one `loading="eager"` (the portrait) and the rest `lazy`.

- [ ] **Step 5: Look at it, in both languages and at both widths**

Serve the built site and open it:

```bash
cd site && zola serve --interface 127.0.0.1 --port 1111
```

Check, at a wide viewport and then at roughly 380 px:
- wide: text left, portrait right, terminal readout spanning underneath;
- narrow: everything centred as before, and the portrait sitting **below** the install button, not above it.

Report what you actually saw. If the portrait appears above the CTA on narrow screens, an `order` property crept in — remove it rather than compensating elsewhere.

- [ ] **Step 6: Commit**

```bash
git add site/templates/index.html site/sass/main.scss
git commit -m "site: the hero gains a second column for Joséphine's portrait"
```

---

### Task 4: The checks section and the footer

Two simple insertions, grouped because neither justifies its own review gate.

**Files:**
- Modify: `site/templates/index.html:201-212` (image above the checks grid)
- Modify: `site/templates/base.html:18-27` (footer image)
- Modify: `site/sass/main.scss` (two new rules)

**Interfaces:**
- Consumes: `josephine-bureau-{768,1024}.webp`, `josephine-repos-{384,512}.webp` from Task 1; the `.art` class from Task 2.
- Produces: nothing.

- [ ] **Step 1: The checks-section image**

In `site/templates/index.html`, insert this figure between the closing `</p>` of `.section__lede` (around line 210) and `<div class="checks">` (line 212):

```html
  <figure class="checks__art">
    <img class="art" src="{{ get_url(path='josephine-bureau-1024.webp') }}"
         srcset="{{ get_url(path='josephine-bureau-768.webp') }} 768w, {{ get_url(path='josephine-bureau-1024.webp') }} 1024w"
         sizes="(max-width: 48rem) 90vw, 34rem"
         width="1024" height="683" loading="lazy" decoding="async"
         alt="{% if fr %}Joséphine devant un ordinateur de bureau, bras ouverts, entourée de flammes violettes{% else %}Joséphine in front of a desktop computer, arms open, surrounded by violet flames{% endif %}">
  </figure>
```

It goes **above** the grid, full width and centred — not beside it. `.checks` is `grid-template-columns: repeat(auto-fit, minmax(15.5rem, 1fr))`, and stealing a column would push the cards below the width their labels need.

- [ ] **Step 2: The footer image**

In `site/templates/base.html`, insert this immediately after `<footer>` (line 18), before `<p class="foot__mark">`:

```html
      <img class="foot__art" src="{{ get_url(path='josephine-repos-512.webp') }}"
           srcset="{{ get_url(path='josephine-repos-384.webp') }} 384w, {{ get_url(path='josephine-repos-512.webp') }} 512w"
           sizes="14rem" width="512" height="512" loading="lazy" decoding="async"
           alt="{% if lang == 'fr' %}Joséphine endormie sur une tour d'ordinateur{% else %}Joséphine asleep on a computer tower{% endif %}">
```

Note the condition is `lang == 'fr'`, **not** `fr` — `base.html` does not define the `fr` shorthand that `index.html` sets at its line 5. Using `fr` here would render neither branch.

- [ ] **Step 3: The two SCSS rules**

Add to `site/sass/main.scss`, after the `.art` rules introduced in Task 2:

```scss
.checks__art { margin: 1.8rem auto 0; max-width: 34rem; }
.foot__art { width: 14rem; height: auto; display: block; margin: 0 auto 1.4rem; opacity: 0.9; }
```

- [ ] **Step 4: Build and assert**

```bash
cd site && zola build && cd ..
grep -c 'josephine-bureau' site/public/index.html site/public/fr/index.html
grep -c 'josephine-repos' site/public/index.html site/public/fr/index.html
grep -o 'Joséphine endormie sur une tour' site/public/fr/index.html
grep -o 'Joséphine asleep on a computer' site/public/index.html
```

Expected: build clean; both images present in both builds; the French alt appears in the French build and the English alt in the English build. If either alt is missing, the `lang == 'fr'` condition in `base.html` was written as `fr` — fix the condition, not the assertion.

- [ ] **Step 5: Commit**

```bash
git add site/templates/index.html site/templates/base.html site/sass/main.scss
git commit -m "site: Joséphine opens the checks section and closes the page"
```

---

### Task 5: Verification pass

Cross-cutting checks that cannot be done inside a single task: the real page weight, both languages end to end, and the audits the spec commits to.

**Files:** none modified unless a check fails.

- [ ] **Step 1: Measure the real page weight**

```bash
cd site && zola build && cd ..
html=$(stat -c%s site/public/index.html)
css=$(cat site/public/*.css 2>/dev/null | wc -c)
imgs=$(for f in josephine-portrait-768 josephine-veille-1024 josephine-bureau-1024 josephine-repos-512; do stat -c%s site/static/$f.webp; done | paste -sd+ | bc)
echo "html $((html/1024)) KB · css $((css/1024)) KB · images (wide) $((imgs/1024)) KB · total $(( (html+css+imgs)/1024 )) KB"
imgs_narrow=$(for f in josephine-portrait-512 josephine-veille-768 josephine-bureau-768 josephine-repos-384; do stat -c%s site/static/$f.webp; done | paste -sd+ | bc)
echo "images (narrow) $((imgs_narrow/1024)) KB"
```

Expected: images around **505 KB** wide and **294 KB** narrow, matching the spec's stated budget. Report the actual numbers. A large deviation means an asset was generated at the wrong width.

- [ ] **Step 2: Confirm only the portrait is eager**

```bash
grep -o 'loading="eager"' site/public/index.html | wc -l
grep -o 'loading="lazy"' site/public/index.html | wc -l
```

Expected: exactly `1` eager and `3` lazy.

- [ ] **Step 3: Confirm no image lacks dimensions**

```bash
grep -o '<img[^>]*>' site/public/index.html site/public/fr/index.html | grep -v 'width=' && echo "FAIL: an img has no width" || echo "all images carry width/height"
```

Expected: the reassurance line. An image without `width`/`height` reflows the page as it loads.

- [ ] **Step 4: Re-audit the shipped assets**

```bash
for f in site/static/josephine-*.webp; do
  w=$(identify -format "%w" $f); h=$(identify -format "%h" $f)
  printf "%-36s %s " "$(basename $f)" "$(identify -format '%[channels]' $f)"
  for xy in "2,2" "$((w-3)),2" "2,$((h-3))" "$((w-3)),$((h-3))"; do
    printf "%s " "$(magick $f -format "%[fx:p{$xy}.a]" info:)"
  done; echo
done
```

Expected: every line `srgba` with four `0` corners.

- [ ] **Step 5: Confirm the copy really was left alone**

```bash
git diff main --stat -- site/content/ site/config.toml
grep -c 'ange gardien' site/content/_index.fr.md site/config.toml
grep -c 'guardian angel' site/content/_index.md site/config.toml
```

Expected: **no** diff under `site/content/` or in `site/config.toml`, and the "ange gardien" / "guardian angel" strings still present and counted. This increment is visual only; if content files changed, something strayed into increment 2's territory.

- [ ] **Step 6: Look at the finished page**

```bash
cd site && zola serve --interface 127.0.0.1 --port 1111
```

Walk both languages top to bottom at a wide viewport and at roughly 380 px. Confirm: the portrait beside the hero text and below the CTA when collapsed; the watching scene where the SVG used to be; the desk scene above the checks grid; Joséphine asleep in the footer; no horizontal scrollbar at any width. Report what you saw, naming anything that looked wrong rather than only what looked right.

- [ ] **Step 7: Commit if anything was fixed**

If steps 1–6 required no change, there is nothing to commit and the branch is ready. If a fix was needed, commit it with a message naming the check that caught it.

---

## Known follow-ups, deliberately out of scope

- **Increment 2, the words:** site copy, `site/config.toml`'s descriptions, the CLI's `about` line at `crates/josephine/src/cli.rs:12,14,110` and the two integration tests pinning it at `crates/josephine/tests/cli.rs:158,171`, both READMEs, `CLAUDE.md`'s "guardian-angel tone" product rule, and new social previews.
- **The favicon stays `✦`** — a detailed chibi at 16 px is unreadable; putting the fox in the tab needs a drawing simplified for that size, which is separate work.
- **`resources/social-preview-{en,fr}.png`** stay PNG whatever happens: GitHub's repository social-preview setting rejects WebP.
- **Merge order.** This branch and `feat/remedies-in-doctor` (PR #37) both touch `site/templates/index.html`. That PR rewrites the critical-notification toast and the CLI showcase; this one restructures the hero and the `.how__art` figure. The edits are in different regions, but whichever merges second will need a look at the other's changes.
