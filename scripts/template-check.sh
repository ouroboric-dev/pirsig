#!/usr/bin/env bash
# template-check — is this repository ready to be copied as a GitHub template?
#
#   scripts/template-check.sh        (or: just template-check)
#
# Offline and quick: each line is one item of docs/template-readiness.md, ✓ or ✗. Exits 1 when
# any item fails. Repository settings (description, topics, issues...) are checked by hand.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"
failed=0
ok() { printf '  ✓ %s\n' "$1"; }
no() { printf '  ✗ %s\n' "$1"; failed=1; }
check() { local what=$1; shift; if "$@" >/dev/null 2>&1; then ok "$what"; else no "$what"; fi; }

echo "License"
check "LICENSE is the MPL-2.0" grep -qx 'Mozilla Public License Version 2.0' LICENSE
check "every crate says MPL-2.0" bash -c '! for m in $(git ls-files "*Cargo.toml"); do grep -q "^\[package\]" "$m" && ! grep -q "^license = \"MPL-2.0\"" "$m" && echo "$m"; done | grep -q .'
check "every package.json says MPL-2.0" bash -c '! for m in $(git ls-files "*package.json"); do grep -q "\"license\": \"MPL-2.0\"" "$m" || echo "$m"; done | grep -q .'

echo "Community files"
for f in README.md LICENSE CONTRIBUTING.md CODE_OF_CONDUCT.md SECURITY.md CHANGELOG.md .editorconfig \
  .github/CODEOWNERS .github/PULL_REQUEST_TEMPLATE.md .github/dependabot.yml \
  .github/ISSUE_TEMPLATE/bug_report.yml .github/ISSUE_TEMPLATE/feature_request.yml .github/ISSUE_TEMPLATE/config.yml; do
  check "$f" test -s "$f"
done
check "README tells how to use the template" grep -q '^## Use this template' README.md
check "README has a License section" grep -q '^## License' README.md

echo "Configuration"
check "pirsig.toml is tracked" git ls-files --error-unmatch pirsig.toml
check "pirsig.toml names the template repository" grep -qE '^repo = "[^"]+/[^"]+"' pirsig.toml
check "the apps target macOS and Android" bash -c \
  'grep -q "\[apps.macos\]" pirsig.toml && grep -q "\[apps.android\]" pirsig.toml && test -d desktop/src-tauri/gen/android'

echo "Nothing private ships"
check "no brain database, env file, key or local config is tracked" bash -c \
  '! git ls-files | grep -E "\.sqlite|(^|/)\.env($|\.)|\.pem$|\.key$|id_(rsa|ed25519)|config/.*\.local\.yaml" | grep -vq "\.env\.example$"'
check "no token or private key in tracked text" bash -c \
  '! git grep -I -n -E "gh[pousr]_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{40,}|sk-ant-[A-Za-z0-9_-]{20,}|AKIA[0-9A-Z]{16}|-----BEGIN [A-Z ]*PRIVATE KEY-----" -- ":!scripts/template-check.sh"'
check "production takes its JWT secret from the environment, with no default" \
  grep -q 'secret: <%= get_env(name="JWT_SECRET") %>' config/production.yaml
check "development's JWT secret can be overridden (JWT_SECRET)" \
  grep -q 'get_env(name="JWT_SECRET", default=' config/development.yaml
check "the project's Claude settings enable no personal plugins" bash -c \
  '! git ls-files --error-unmatch .claude/settings.json 2>/dev/null || ! grep -q enabledPlugins .claude/settings.json'
check ".claude/settings.local.json is ignored" git check-ignore -q --no-index .claude/settings.local.json
check "brain/ holds no #private note" bash -c \
  '! grep -rlE "^tags: .*\"private\"" brain'
check "brain/ names no private repository (no '(private)' lines)" bash -c '! grep -rl "(private)" brain'

echo "CI"
check "every action is pinned to a commit" bash -c \
  '! grep -hE "^\s*(- )?uses:" .github/workflows/* | grep -vE "@[0-9a-f]{40}( |$)"'
check "every workflow declares its permissions" bash -c \
  '! for w in .github/workflows/*; do grep -q "permissions:" "$w" || echo "$w"; done | grep -q .'

echo "Adoption"
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
git ls-files -z | xargs -0 tar cf - 2>/dev/null | tar xf - -C "$tmp"
if (cd "$tmp" && git init -q && git add -A && scripts/adopt.sh acme brain >/dev/null); then
  ok "just adopt acme brain runs in a copy"
  check "after it, no URL of h0ffmann/pirsig is left outside brain/, examples/ and tests/" bash -c \
    "cd '$tmp' && ! git grep -q 'h0ffmann/pirsig' -- ':!brain' ':!examples' ':!tests' ':!*.lock' ':!CHANGELOG.md' ':!scripts'"
  check "after it, the app id is io.github.acme.brain" grep -q '"io.github.acme.brain"' "$tmp/desktop/src-tauri/tauri.conf.json"
  check "after it, CODEOWNERS is @acme" grep -q '@acme' "$tmp/.github/CODEOWNERS"
  check "after it, pirsig.toml bootstraps from acme/brain" grep -qx 'repo = "acme/brain"' "$tmp/pirsig.toml"
  check "after it, the Android app is io.github.acme.brain" bash -c \
    "cd '$tmp' && ! git grep -qF 'io.github.h0ffmann.pirsig' -- ':!scripts' && grep -q 'io.github.acme.brain' desktop/src-tauri/gen/android/app/build.gradle.kts"
else
  no "just adopt acme brain runs in a copy"
fi

if [ "$failed" -eq 0 ]; then echo "template-check: ready"; else echo "template-check: not ready" >&2; fi
exit "$failed"
