#!/usr/bin/env bash
# Tile frames from a video into one PNG, so you can check a whole animation at a glance.
#
#   contact-sheet.sh VIDEO [SECONDS_BETWEEN_FRAMES=1.5] [COLUMNS=4] [OUT.png] [FRAME_WIDTH=360]
#
# Frames run left to right, top to bottom. Look for: things overlapping or clipped at the
# edges, text colliding with shapes, something that never appears, an unfinished last frame.
set -euo pipefail

if [ $# -lt 1 ]; then
  echo "usage: contact-sheet.sh VIDEO [SECONDS_BETWEEN_FRAMES] [COLUMNS] [OUT.png] [FRAME_WIDTH]" >&2
  exit 2
fi
video=$1
step=${2:-1.5}
cols=${3:-4}
out=${4:-contact-sheet.png}
width=${5:-360}
max_rows=6

for tool in ffmpeg ffprobe; do
  command -v "$tool" >/dev/null || { echo "contact-sheet.sh: $tool is not installed (brew install ffmpeg)" >&2; exit 1; }
done
[ -f "$video" ] || { echo "contact-sheet.sh: no such file: $video" >&2; exit 1; }

duration=$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$video")
# frames = ceil(duration / step), rows = ceil(frames / cols), capped so the sheet stays readable.
read -r frames rows < <(awk -v d="$duration" -v s="$step" -v c="$cols" -v m="$max_rows" 'BEGIN {
  f = int(d / s); if (f < d / s) f++; if (f < 1) f = 1;
  r = int(f / c); if (r < f / c) r++; if (r > m) { r = m; f = r * c }
  print f, r }')

ffmpeg -v error -y -i "$video" \
  -vf "fps=1/$step,scale=$width:-2,tile=${cols}x${rows}" -frames:v 1 "$out"
[ -s "$out" ] || { echo "contact-sheet.sh: could not build the sheet" >&2; exit 1; }
echo "$out (up to $frames frames, one every ${step}s, of ${duration}s)"
