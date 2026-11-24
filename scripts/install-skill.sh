#!/usr/bin/env bash
# Install the rough-animations skill into an AI agent.
#
#   curl -fsSL https://raw.githubusercontent.com/Super-Yojan/rough-animations/main/scripts/install-skill.sh | bash
#   curl -fsSL https://raw.githubusercontent.com/Super-Yojan/rough-animations/main/scripts/install-skill.sh | bash -s -- --project
#
# Where it goes:
#   (default)    for every project: $CLAUDE_CONFIG_DIR/skills, or ~/.claude/skills
#   --project    this project only: ./.claude/skills
#   --dir PATH   PATH/rough-animations, for any agent that reads SKILL.md folders
#
# Other options:
#   --ref NAME   the git branch or tag to download (default: main)
#   --from DIR   install from a checkout of the repository instead of downloading
#   --uninstall  remove the skill from the chosen place
#   -h, --help   show this help
#
# Needs bash, plus curl and tar when it downloads. Running it again updates the skill in place.
set -euo pipefail

NAME=rough-animations
REPO=${ROUGH_ANIMATIONS_REPO:-Super-Yojan/rough-animations}
REF=main
FROM=""
MODE=global
DIR=""
UNINSTALL=0

usage() { sed -n '2,/^set -euo/p' "$0" | sed '$d' | sed 's/^# \{0,1\}//'; }
die() { echo "install-skill: $*" >&2; exit 1; }

while [ $# -gt 0 ]; do
  case "$1" in
    --global)    MODE=global ;;
    --project)   MODE=project ;;
    --dir)       [ $# -ge 2 ] || die "--dir needs a path"; MODE=dir; DIR=$2; shift ;;
    --ref)       [ $# -ge 2 ] || die "--ref needs a branch or tag"; REF=$2; shift ;;
    --from)      [ $# -ge 2 ] || die "--from needs a directory"; FROM=$2; shift ;;
    --uninstall) UNINSTALL=1 ;;
    -h|--help)   if [ -f "$0" ]; then usage; else echo "see the header of scripts/install-skill.sh"; fi; exit 0 ;;
    *)           echo "install-skill: unknown option: $1" >&2; echo "try --help" >&2; exit 2 ;;
  esac
  shift
done

# --- where the skills folder is ---------------------------------------------------------------
case "$MODE" in
  global)  target=${CLAUDE_CONFIG_DIR:-$HOME/.claude}/skills ;;
  project) target=$PWD/.claude/skills ;;
  dir)     [ -n "$DIR" ] || die "--dir needs a path"; target=$DIR ;;
esac
[ -n "$target" ] && [ "$target" != "/" ] || die "refusing to use '$target' as the skills folder"
dest=$target/$NAME

# --- uninstall --------------------------------------------------------------------------------
if [ "$UNINSTALL" = 1 ]; then
  if [ ! -e "$dest" ]; then
    echo "Not installed at $dest"
    exit 0
  fi
  # Only ever delete something that really is this skill.
  grep -q "^name: $NAME\$" "$dest/SKILL.md" 2>/dev/null \
    || die "$dest does not look like the $NAME skill, so I won't delete it"
  rm -rf "$dest"
  echo "Removed $dest"
  exit 0
fi

# --- find the skill to install ----------------------------------------------------------------
tmp=$(mktemp -d)
staging=""
trap 'rm -rf "$tmp" ${staging:+"$staging"}' EXIT

src=""
if [ -n "$FROM" ]; then
  src=$FROM/skills/$NAME
elif [ -n "${BASH_SOURCE[0]:-}" ] && [ -f "${BASH_SOURCE[0]}" ]; then
  # Run from a checkout: use the skill next to this script.
  here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
  [ -f "$here/../skills/$NAME/SKILL.md" ] && src=$here/../skills/$NAME
fi

if [ -z "$src" ]; then
  command -v curl >/dev/null || die "curl is needed to download the skill (or use --from a checkout)"
  command -v tar  >/dev/null || die "tar is needed to unpack the skill"
  url=${ROUGH_ANIMATIONS_TARBALL:-https://github.com/$REPO/archive/$REF.tar.gz}
  echo "Downloading $url"
  curl -fsSL "$url" | tar -xz -C "$tmp" \
    || die "could not download or unpack $url (is the repository public and pushed, and does '$REF' exist?)"
  src=$(ls -d "$tmp"/*/skills/$NAME 2>/dev/null | head -n 1 || true)
  [ -n "$src" ] || die "the download has no skills/$NAME folder (wrong repository or branch?)"
fi
[ -f "$src/SKILL.md" ] || die "no SKILL.md in $src"

# --- install, replacing any older copy --------------------------------------------------------
mkdir -p "$target"
staging=$(mktemp -d "$target/.$NAME.XXXXXX")
cp -R "$src/." "$staging/"
[ -d "$staging/scripts" ] && chmod +x "$staging"/scripts/*.sh
rm -rf "$dest"
mv "$staging" "$dest"
staging=""

echo "Installed the $NAME skill to $dest"
echo "Start a new agent session to pick it up."
