#!/usr/bin/env bash
# Save one frame of a video as a PNG, so you can look at it with an image viewer.
#
#   frame.sh VIDEO TIME_SECONDS [OUT.png]
#
# TIME_SECONDS may be negative to count back from the end (-0.5 is half a second before it
# finishes), which gives the finished scene.
set -euo pipefail

if [ $# -lt 2 ]; then
  echo "usage: frame.sh VIDEO TIME_SECONDS [OUT.png]" >&2
  exit 2
fi
video=$1
time=$2
out=${3:-frame.png}

if ! command -v ffmpeg >/dev/null; then
  echo "frame.sh: ffmpeg is not installed (brew install ffmpeg)" >&2
  exit 1
fi
[ -f "$video" ] || { echo "frame.sh: no such file: $video" >&2; exit 1; }

case "$time" in
  -*) seek=(-sseof "$time") ;;
  *)  seek=(-ss "$time") ;;
esac

ffmpeg -v error -y "${seek[@]}" -i "$video" -frames:v 1 "$out"
[ -s "$out" ] || { echo "frame.sh: no frame at $time s (is it past the end?)" >&2; exit 1; }
echo "$out"
