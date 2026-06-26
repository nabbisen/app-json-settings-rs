#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

for dir in rfcs/proposed rfcs/done rfcs/archive; do
  if [[ ! -d "$dir" ]]; then
    echo "missing RFC state directory: $dir" >&2
    exit 1
  fi
done

numbers="$(find rfcs/proposed rfcs/done rfcs/archive -type f -name '[0-9][0-9][0-9]-*.md' -printf '%f\n' | cut -d- -f1 | sort)"
duplicates="$(printf '%s\n' "$numbers" | uniq -d)"
if [[ -n "$duplicates" ]]; then
  echo "duplicate RFC number(s):" >&2
  printf '%s\n' "$duplicates" >&2
  exit 1
fi

while IFS= read -r file; do
  case "$file" in
    rfcs/proposed/*) expected="Proposed" ;;
    rfcs/done/*) expected="Implemented" ;;
    rfcs/archive/*) expected="Withdrawn\|Superseded" ;;
    *) continue ;;
  esac

  if ! grep -Eq "^\*\*Status\.\*\* ($expected)" "$file"; then
    echo "RFC status does not match folder: $file" >&2
    exit 1
  fi
done < <(find rfcs/proposed rfcs/done rfcs/archive -type f -name '[0-9][0-9][0-9]-*.md' | sort)

for file in $(find rfcs/proposed rfcs/done rfcs/archive -type f -name '[0-9][0-9][0-9]-*.md' | sort); do
  if ! grep -q "$(basename "$file")" rfcs/README.md && ! grep -q "$(basename "$file" .md)" rfcs/README.md; then
    echo "RFC missing from index: $file" >&2
    exit 1
  fi
done
