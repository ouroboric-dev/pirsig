#!/usr/bin/env bash
# adopt — make a copy of the Pirsig template yours: point its identity at OWNER/REPO.
#
#   scripts/adopt.sh OWNER [REPO]     (or: just adopt OWNER [REPO]; REPO defaults to this folder)
#
# Rewrites the repository's own URL (README badges and clone lines, the plugin, the GitHub App
# manifest, the importers' user agent), the desktop app's id, CODEOWNERS and the contacts in
# SECURITY.md and CODE_OF_CONDUCT.md. It leaves brain/ (the author's notes), examples/ and
# tests/ alone, and the credits to the author's other repositories. Review with `git diff`.
set -euo pipefail

owner=${1:?usage: scripts/adopt.sh OWNER [REPO]}
root=$(git rev-parse --show-toplevel)
repo=${2:-$(basename "$root")}
[[ $owner =~ ^[A-Za-z0-9-]+$ && $repo =~ ^[A-Za-z0-9._-]+$ ]] ||
  { echo "adopt: OWNER and REPO must be GitHub names, got '$owner' '$repo'" >&2; exit 2; }
cd "$root"

# a reverse-DNS id: lowercase letters, digits and hyphens
id() { tr '[:upper:]' '[:lower:]' <<<"$1" | tr -c 'a-z0-9\n' '-'; }

# the repository's own URL, wherever it is written
# (a while loop, not mapfile: macOS ships bash 3.2)
count=0
while IFS= read -r f; do
  perl -pi -e "s{h0ffmann/pirsig}{$owner/$repo}g" "$f"
  count=$((count + 1))
done < <(git grep -l 'h0ffmann/pirsig' -- ':!brain' ':!examples' ':!tests' \
  ':!*.lock' ':!CHANGELOG.md' ':!scripts/adopt.sh' ':!scripts/template-check.sh' || true)

# the app id, who owns it, who reviews it, whom to tell
perl -pi -e "s/io\.github\.h0ffmann\.pirsig/io.github.$(id "$owner").$(id "$repo")/" \
  desktop/src-tauri/tauri.conf.json desktop/README.md
# the Android project: a package name takes no '-', so '_' (as Tauri does), and the
# Kotlin sources move to the package's folders
pkg="io.github.$(id "$owner" | tr - _).$(id "$repo" | tr - _)"
android=desktop/src-tauri/gen/android
while IFS= read -r f; do
  perl -pi -e "s/io\.github\.h0ffmann\.pirsig/$pkg/g" "$f"
done < <(git grep -l 'io\.github\.h0ffmann\.pirsig' -- "$android" || true)
for src in "$android/app/src/main/java" "$android/buildSrc/src/main/java"; do
  if [ -d "$src/io/github/h0ffmann/pirsig" ]; then
    to="$src/$(tr . / <<<"$pkg")"
    mkdir -p "$(dirname "$to")"
    mv "$src/io/github/h0ffmann/pirsig" "$to"
    rmdir "$src/io/github/h0ffmann" 2>/dev/null || true
  fi
done
perl -pi -e "s/\"name\": \"h0ffmann\"/\"name\": \"$owner\"/" \
  .claude-plugin/marketplace.json plugins/pirsig/.claude-plugin/plugin.json
perl -pi -e "s{\"author\": \"h0ffmann\"}{\"author\": \"$owner\"}; s{\"authorUrl\": \"https://github.com/h0ffmann\"}{\"authorUrl\": \"https://github.com/$owner\"}" \
  obsidian-painter/manifest.json
perl -pi -e "s{\@h0ffmann\b}{\@$owner}g; s{github\.com/h0ffmann\)}{github.com/$owner)}g" \
  .github/CODEOWNERS SECURITY.md CODE_OF_CONDUCT.md

echo "adopt: this copy is now $owner/$repo ($count files with its URL). Next:"
echo "  git diff                       # review, then commit"
echo "  just brain-reset               # your own GitHub in the brain (or: just fork-state)"
echo "  brain/ holds h0ffmann's public notes: keep them, or git rm -r brain and later just fork-export"
