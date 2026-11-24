#!/usr/bin/env bash
# Copy docs/cheatsheet.md into the API section of the agent skill, so the two never drift.
# Run it after editing the cheat sheet; `cargo test` fails if you forget.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
sheet=$root/docs/cheatsheet.md
skill=$root/skills/rough-animations/SKILL.md

grep -q '<!-- cheatsheet:start -->' "$skill" && grep -q '<!-- cheatsheet:end -->' "$skill" \
  || { echo "sync-skill.sh: the markers are missing from $skill" >&2; exit 1; }

tmp=$(mktemp)
awk -v sheet="$sheet" '
  /<!-- cheatsheet:start -->/ { print; while ((getline line < sheet) > 0) print line; skip = 1; next }
  /<!-- cheatsheet:end -->/   { skip = 0 }
  !skip                       { print }
' "$skill" > "$tmp"
mv "$tmp" "$skill"
echo "updated $skill from $sheet"
