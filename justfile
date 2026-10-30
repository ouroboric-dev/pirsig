# psg tasks. `just` lists them; `just <name> args...` runs one.
# Rust does the work, just only remembers the commands.

set dotenv-load := true

# rustup's cargo first, so rust-toolchain.toml wins over a Homebrew or distro cargo
export PATH := env("CARGO_HOME", home_directory() / ".cargo") / "bin:" + env("PATH")

psg := "cargo run -q --bin psg --"

default:
    @just --list --unsorted

# --- daily use -------------------------------------------------------------

# Capture a fleeting note: just n "An idea" [-b "more"] [--tag x]
n title *args:
    {{psg}} note -c "{{title}}" {{args}}

# Continue a train of thought: just after 21 "Next idea"
after zid title *args:
    {{psg}} note -c "{{title}}" -a {{zid}} {{args}}

# Show a note with links and backlinks
show zid:
    {{psg}} note -s {{zid}}

# Edit a note's body in $EDITOR
edit zid:
    {{psg}} note -u {{zid}} -e

# Search titles and bodies
find query:
    {{psg}} note -q "{{query}}"

# Link two notes with a reason: just link 21a 3 "contradicts"
link from to why="":
    {{psg}} link {{from}} {{to}} -w "{{why}}"

# Slipbox views: stats | inbox | orphans | random | tree | tags | graph
box view="stats":
    {{psg}} box {{view}}

# Export every note as Markdown into a folder (default ./vault)
export dir="vault":
    {{psg}} box export {{dir}}

# Run any Pirsig command: just psg para --json
psg *args:
    {{psg}} {{args}}

# Install Pirsig on your PATH
install:
    cargo install --path . --bin psg --locked

# --- development -----------------------------------------------------------

# API server on http://localhost:5150
dev:
    cargo loco start

# Methodology readiness: PARA, Zettelkasten, Obsidian, RSS, arXiv
ready:
    {{psg}} ready

# Seed the brain from your GitHub repos: just bootstrap --private minimal --readme
bootstrap *args:
    GH_TOKEN=$(gh auth token) {{psg}} bootstrap gh {{args}}

# Start from any Pirsig repository's brain/ (default: [template] repo in pirsig.toml): just bootstrap-repo acme/brain
bootstrap-repo *args:
    {{psg}} bootstrap repo {{args}}

# Start from the fork state in brain/ (h0ffmann's public notes); replaces your brain after a backup
fork-state:
    {{psg}} fork import brain --overwrite

# Replace the whole brain with your own GitHub (backed up first): just brain-reset --readme
brain-reset *args:
    GH_TOKEN=$(gh auth token) {{psg}} gh --overwrite --private minimal {{args}}

# Make a copy of the template yours: just adopt <owner> [<repo>] (URLs, app id, owners, contacts)
adopt owner repo="":
    scripts/adopt.sh {{owner}} {{repo}}

# Is this repository ready to be copied as a GitHub template? (offline; see docs/template-readiness.md)
template-check:
    scripts/template-check.sh

# Write your brain to brain/, without #private notes: the fork state others start from
fork-export:
    {{psg}} fork export brain

# A quick Plasticity session: five exercises
plast:
    {{psg}} plast go

# Build the obsidian-painter plugin (Rust core to WebAssembly, then main.js) and test it
painter:
    cd obsidian-painter && npm ci && npm run build && npm run check && npm test

# Open the app (Tauri) on macOS: the whole brain, on the same database as `psg`
desktop:
    cargo run --manifest-path desktop/src-tauri/Cargo.toml

# Build Pirsig.app for macOS (desktop/src-tauri/target/release/bundle/macos); unsigned
desktop-build:
    cd desktop && npm ci && npx tauri build --bundles app,dmg

# Run the app on an Android device or emulator (needs the Android SDK, NDK and JDK 17)
android-dev:
    cd desktop && npm ci && npx tauri android dev

# Build an Android APK for arm64 phones (desktop/src-tauri/gen/android/app/build/outputs)
android:
    cd desktop && npm ci && npx tauri android build --apk --target aarch64

# Build the phone UI (crates/pirsig-mobile, Leptos to WebAssembly) into desktop/dist/mobile;
# then `just dev` serves it at http://localhost:5150/mobile/
mobile-ui *args:
    scripts/build-mobile-ui.sh {{args}}

# The UI end to end in WebKit, on a fresh throwaway brain (Playwright)
desktop-e2e:
    cd desktop && npm ci && npx playwright install webkit && npx playwright test

# Format, lint and test: what CI runs
check: fmt-check lint test

# Alias for `check`: the marola-devkit stop hook asks for `just quality`
quality: check

test *args:
    cargo test --all-features --all {{args}}

lint:
    cargo clippy --all-features -- -D warnings -W clippy::pedantic -W clippy::nursery -W rust-2018-idioms

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

# Apply pending migrations and regenerate src/models/_entities
migrate:
    cargo loco db migrate
    cargo loco db entities

# New model with migration: just model notes title:string!
model name *fields:
    cargo loco generate model {{name}} {{fields}}

# Drop the development database and start over
reset:
    cargo loco db reset

# --- figures and graphs ----------------------------------------------------

# Check out the pinned Archify (vendor/archify, sparse to archify/), once per clone
figure-init:
    git submodule update --init --depth 1 vendor/archify
    git -C vendor/archify sparse-checkout set --no-cone /archify/
    @echo "figure-init: $(git -C vendor/archify rev-parse --short HEAD)"

# Archify validates and checks each source (finalize), picasso's look (docs/archify/picasso.css) goes
# over it, headless Chrome takes the picture. Needs node, jq, perl, Chrome (CHROME names another) and
# the Google Fonts; writes docs/archify/figures.lock.
# Draw every docs/archify/<slug>.json to docs/img/<slug>.png
figure: figure-init
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p build/figure docs/img
    for src in docs/archify/*.json; do
        slug="$(basename "$src" .json)"; out="build/figure/$slug"
        rm -rf "$out"; mkdir -p "$out"
        ARCHIFY_UPDATE_CHECK_DISABLED=1 node vendor/archify/archify/bin/archify.mjs finalize "$(jq -r .diagram_type "$src")" "$src" "$out/page.html" --out-dir "$out/receipts" --quality "$(jq -r '.meta.quality_profile // "showcase"' "$src")"
        read -r w h < <(grep -o '<svg viewBox="[^"]*"' "$out/page.html" | head -1 | sed -E 's/.*0 0 ([0-9.]+) ([0-9.]+)"/\1 \2/')
        crop="$(awk -F'\t' -v s="$slug" '$1==s {print $2}' docs/archify/crop.tsv)"; h="${crop:-$h}"
        FONTS='<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Fjalla+One&family=Inter:wght@400;600&display=block">' CSS="$(cat docs/archify/picasso.css)" W="$w" H="$h" \
            perl -0pe 's|</head>|$ENV{FONTS}<style>$ENV{CSS}</style></head>|; s|<svg viewBox="0 0 [0-9.]+ [0-9.]+"|<svg viewBox="0 0 $ENV{W} $ENV{H}"|' "$out/page.html" > "$out/frame.html"
        "${CHROME:-google-chrome}" --headless --disable-gpu --hide-scrollbars --force-dark-mode --force-device-scale-factor=2 --user-data-dir="$PWD/$out/profile" --window-size="$w,$h" --virtual-time-budget=3000 --screenshot="$PWD/docs/img/$slug.png" "file://$PWD/$out/frame.html" 2>/dev/null
        echo "figure: docs/img/$slug.png"
    done
    just _figure-lock > docs/archify/figures.lock

# The demo brain lives in a throwaway SQLite file, so the PARA picture is the CLI's own output;
# `just figure` then draws it.
# Write docs/archify/para.json with `psg figure` from a demo brain
figure-para:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p build/figure
    db="$PWD/build/figure/demo.sqlite"; rm -f "$db"
    export DATABASE_URL="sqlite://$db?mode=rwc"
    cargo build -q --bin psg
    z=target/debug/psg
    $z area -c -n Health --description "body and mind" >/dev/null
    $z area -c -n Work >/dev/null
    $z area -c -n "Second brain" >/dev/null
    $z project -c -n "Run a marathon" -a 1 --status active >/dev/null
    $z project -c -n "Q4 roadmap" -a 2 --status waiting >/dev/null
    $z project -c -n "Ship Pirsig v0" -a 3 --status active >/dev/null
    $z task -c -t "Buy shoes" -p 1 --due 2026-11-01 >/dev/null
    $z task -c -t "Long run, 21 km" -p 1 >/dev/null
    $z task -c -t "Obsidian sync" -p 3 >/dev/null
    $z task -c -t "Archify figures" -p 3 >/dev/null
    $z resource -c -t "Couch to 5k" --url https://example.com/c25k -p 1 >/dev/null
    $z resource -c -t "PARA method" -a 3 >/dev/null
    $z resource -c -t "Reading list" >/dev/null
    $z figure -t "A second brain in Pirsig" -o docs/archify/para.json

# The inputs of each figure (its source, the look, the crop, the Archify pin), one line per slug.
_figure-lock:
    #!/usr/bin/env bash
    set -euo pipefail
    pin="$(git ls-files -s vendor/archify | cut -d' ' -f2)"
    for src in docs/archify/*.json; do
        printf '%s\t%s\n' "$(basename "$src" .json)" "$(cat "$src" docs/archify/picasso.css docs/archify/crop.tsv | sha256sum | cut -c1-16)-${pin:0:7}"
    done

# Cheap and offline: every source has its PNG, README.md shows it, both match the last `just figure`
figure-check:
    #!/usr/bin/env bash
    set -euo pipefail
    for src in docs/archify/*.json; do
        s="$(basename "$src" .json)"
        test -s "docs/img/$s.png" || { echo "figure-check: no PNG for $src; run just figure" >&2; exit 1; }
        grep -qF "](docs/img/$s.png)" README.md || { echo "figure-check: README.md does not show docs/img/$s.png" >&2; exit 1; }
    done
    just _figure-lock | cmp -s - docs/archify/figures.lock || { echo "figure-check: a source, the look or the Archify pin changed since the PNG; run just figure" >&2; exit 1; }
    echo "figure-check: ok"

# Writes graphify-out/graph.json, then communities, GRAPH_REPORT.md and graph.html. Needs graphify
# on PATH (pip install graphifyy==0.9.96); DATABASE_URL picks the brain.
# The brain as a graphify knowledge graph
graph:
    {{psg}} graph -o graphify-out/graph.json
    graphify cluster-only . --no-label
    @echo "graph: graphify-out/graph.html; ask with: graphify query \"...\""
