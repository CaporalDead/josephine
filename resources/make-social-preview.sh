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
