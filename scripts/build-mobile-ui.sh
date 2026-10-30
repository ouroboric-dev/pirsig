#!/usr/bin/env bash
# build-mobile-ui — the Android app's page: crates/pirsig-mobile as WebAssembly, in desktop/dist/mobile/
#
#   scripts/build-mobile-ui.sh [--debug]        (or: just mobile-ui)
#
# Needs the wasm32 target (rust-toolchain.toml installs it) and the wasm-bindgen CLI of the
# version crates/pirsig-mobile pins: cargo install wasm-bindgen-cli --version 0.2.129 --locked.
# Tauri runs this before it builds the app (desktop/src-tauri/tauri.conf.json), so the page is
# in desktop/dist when the app embeds it. The output is not committed.
set -euo pipefail
cd "$(dirname "$0")/.."
# rustup's cargo first, so rust-toolchain.toml wins over a Homebrew or distro cargo (as in the justfile)
export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"

profile=release
flags=(--release)
if [[ ${1:-} == --debug ]]; then
  profile=debug
  flags=()
fi

want=$(sed -n 's/^wasm-bindgen = { version = "=\([0-9.]*\)" }$/\1/p' crates/pirsig-mobile/Cargo.toml)
have=$(wasm-bindgen --version 2>/dev/null | awk '{print $2}' || true)
if [[ $have != "$want" ]]; then
  echo "build-mobile-ui: needs wasm-bindgen $want (found ${have:-none}):" >&2
  echo "  cargo install wasm-bindgen-cli --version $want --locked" >&2
  exit 1
fi

cargo build --locked -p pirsig-mobile --target wasm32-unknown-unknown "${flags[@]}"

out=desktop/dist/mobile
rm -rf "$out"
mkdir -p "$out"
wasm-bindgen --target web --no-typescript --out-dir "$out/pkg" \
  "target/wasm32-unknown-unknown/$profile/pirsig_mobile.wasm"
cp crates/pirsig-mobile/index.html crates/pirsig-mobile/boot.js "$out/"
cp crates/pirsig-mobile/style/tokens.css crates/pirsig-mobile/style/app.css "$out/"
cp -R crates/pirsig-mobile/fonts "$out/fonts"
echo "build-mobile-ui: $out ($(du -sh "$out" | cut -f1))"
