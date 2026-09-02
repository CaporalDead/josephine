#!/usr/bin/env bash
# Compose Joséphine's GitHub social preview — 1200x630 PNG, one per language.
#
# The text is rendered by the font engine rather than generated, so it cannot
# come out garbled. Regenerate after any change to the formula:
#   ./resources/make-social-preview.sh en && ./resources/make-social-preview.sh fr
set -euo pipefail

lang="${1:?usage: make-social-preview.sh <en|fr>}"
case "$lang" in
  en) formula="Your computer's guardian angel" ;;
  fr) formula="L'ange gardien de votre ordinateur" ;;
  *)  echo "unknown language: $lang (expected en or fr)" >&2; exit 1 ;;
esac

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
out="$root/resources/social-preview-$lang.png"

# ImageMagick treats an unresolvable font as a warning, not an error, and
# still writes the file in a fallback font — which `set -e` cannot catch.
# Note: intentionally `grep -x` rather than `grep -qx` — with `pipefail` set,
# `-q` makes grep exit as soon as it finds a match, which can SIGPIPE the
# still-writing `fc-list`/`tr` upstream and fail the pipeline even on a match.
fc-list : family | tr ',' '\n' | grep -x "Source Code Pro" >/dev/null ||
  { echo "Source Code Pro is not installed — the wordmark would silently change font" >&2; exit 1; }

night="#0b0e19"
star_col="#e9eaf6"
violet="#9a86e0"
violet_bright="#b3a3ec"
dim="#cdd2ea"
mono="Source-Code-Pro"

# The ✦ is drawn from a path, not typed: Source Code Pro has no glyph for
# U+2726, and the fonts that do (Symbola, FreeSerif) are not dependable on
# another machine. A path renders identically everywhere.
#
# The halo above it is what makes the mark Joséphine's rather than a generic
# star — the same ellipse the site's guardian() macro draws over the laptop.
mark_svg=$(mktemp --suffix=.svg)
trap 'rm -f "$mark_svg"' EXIT
cat > "$mark_svg" <<SVG
<svg xmlns="http://www.w3.org/2000/svg" width="128" height="128" viewBox="0 0 44 44">
  <ellipse cx="22" cy="7" rx="13" ry="3.4" fill="none"
           stroke="$violet_bright" stroke-opacity="0.80" stroke-width="1.1"/>
  <ellipse cx="22" cy="7" rx="13" ry="3.4" fill="none"
           stroke="$violet" stroke-opacity="0.16" stroke-width="4"/>
  <path d="M22 14 L24.4 24.6 L35 27 L24.4 29.4 L22 40 L19.6 29.4 L9 27 L19.6 24.6 Z"
        fill="$violet_bright"/>
</svg>
SVG

# Night ground, then a restrained violet glow, then the mark and the wordmark,
# centred. Two things the glow layer has to get right: it is built on black,
# not on $night, because the Screen composite must only *add* light where the
# glow actually is — screening a $night layer lifts the entire canvas and
# washes the formula out; and it is damped afterwards so the ground still
# reads as night. Joséphine's signature is the haloed ✦, not a portrait — the
# illustration this used to composite belonged to the guardian-spirit identity
# and went out with it.
#
# `-background none` before the SVG is load-bearing: ImageMagick otherwise
# rasterises it over opaque white, which lands the mark in a white box.
magick -size 1200x630 "xc:$night" \
  \( -size 1200x630 "xc:black" \
     -fill "$violet" -draw 'circle 600,255 600,75' -blur 0x130 \
     -evaluate Multiply 0.62 \) \
  -compose Screen -composite \
  -compose Over \
  -background none "$mark_svg" -gravity north -geometry +0+130 -composite \
  -font "$mono" -fill "$star_col" -pointsize 92 \
  -gravity north -annotate +0+288 "Joséphine" \
  -font "$mono" -fill "$dim" -pointsize 27 \
  -gravity north -annotate +0+414 "$formula" \
  -font "$mono" -fill "$violet" -pointsize 19 \
  -gravity south -annotate +0+72 "github.com/systm-d/josephine" \
  -depth 8 -alpha off -strip \
  "PNG24:$out"

echo "wrote $out ($(identify -format '%wx%h' "$out"), $(du -h "$out" | cut -f1))"
