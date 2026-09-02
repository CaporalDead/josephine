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
dim="#8b91b4"
mono="Source-Code-Pro"

# The ✦ is drawn from a path, not typed: Source Code Pro has no glyph for
# U+2726, and the fonts that do (Symbola, FreeSerif) are not dependable on
# another machine. A path renders identically everywhere.
star_svg=$(mktemp --suffix=.svg)
trap 'rm -f "$star_svg"' EXIT
cat > "$star_svg" <<SVG
<svg xmlns="http://www.w3.org/2000/svg" width="88" height="88" viewBox="0 0 24 24">
  <path d="M12 0 L14.4 9.6 L24 12 L14.4 14.4 L12 24 L9.6 14.4 L0 12 L9.6 9.6 Z" fill="$violet"/>
</svg>
SVG

# Night ground, then a soft violet glow, then the mark and the wordmark,
# centred. Joséphine's signature is the ✦, not a portrait — the illustration
# it used to composite belonged to the guardian-spirit identity.
magick -size 1200x630 "xc:$night" \
  \( -size 1200x630 "xc:$night" \
     -fill "$violet" -draw 'circle 600,315 600,120' -blur 0x70 \) \
  -compose Screen -composite \
  -compose Over \
  "$star_svg" -gravity north -geometry +0+150 -composite \
  -font "$mono" -fill "$star_col" -pointsize 88 \
  -gravity north -annotate +0+268 "Joséphine" \
  -font "$mono" -fill "$dim" -pointsize 26 \
  -gravity north -annotate +0+392 "$formula" \
  -font "$mono" -fill "$violet" -pointsize 19 \
  -gravity south -annotate +0+72 "github.com/systm-d/josephine" \
  -depth 8 -alpha off -strip \
  "PNG24:$out"

echo "wrote $out ($(identify -format '%wx%h' "$out"), $(du -h "$out" | cut -f1))"
