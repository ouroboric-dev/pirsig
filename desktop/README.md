# Pirsig app (Tauri): macOS and Android

`just desktop` opens Pirsig as a desktop app built with [Tauri 2](https://v2.tauri.app); the
same app runs on Android. Those are its two targets for now. The whole brain is managed from
it:

| Section | What you can do |
|---|---|
| Today | stats, readiness score, the inbox, active projects, dated tasks; quick capture from the sidebar on every page |
| Notes | search by text, kind and tag; write and edit with a live preview (`[[links]]`, `#tags`, ⌘S); link with a reason, unlink, continue a note (Folgezettel), pull into a project, see the Markdown Obsidian gets, delete |
| Slip-box | inbox, orphans, the Folgezettel tree, tags, a random note |
| Hippos | create, edit, delete; the rhizome map (plateaus, pieces, where you left off); grow a note in it; weave |
| Projects | create, edit, archive, delete (with their tasks, after a second confirmation); tasks; the notes a project pulls in, with why; its resources |
| Areas | create, edit, archive, delete (with their projects, after a second confirmation); an area's projects and resources |
| Resources | create, edit, filter, archive, delete; write a literature note from one |
| Tasks | open and done, across projects; tick, edit, delete |
| Feeds and arXiv | subscribe, pull, unsubscribe; search arXiv and ingest papers as resources |
| Plasticity, Readiness | practise exercises; the methodology checks with their fixes |

## How it fits together

```
pirsig-desktop (Tauri)                        the same in a browser
  ├─ Pirsig's REST API (Loco), 127.0.0.1:<free port>     `just dev` → localhost:5150
  │    └─ serves desktop/dist (the UI) at /, and the phone UI at /mobile/
  └─ a window on http://127.0.0.1:<port>/ (macOS) or …/mobile/ (Android)
```

The app starts the Loco app from the `zettelkasten` crate in-process, on a free port of
`127.0.0.1`, and opens its window on it. The API also serves the page (`config/development.yaml`,
`server.middlewares.static`), so the page and the API share an origin: no CORS, no Tauri
commands, and the same UI in a browser while `just dev` runs. The page is plain HTML, CSS
and ES modules (`dist/`), with no build step and no dependencies; it builds the DOM from text,
so nothing a note holds is ever parsed as HTML.

## The phone UI

On Android the window opens `/mobile/`: a UI made for one hand, written in Rust with
[Leptos](https://leptos.dev) and compiled to WebAssembly (`crates/pirsig-mobile`). It talks to
the same API, so a phone and a laptop see the same brain. It follows the mobile design drawn in
[OpenPencil](https://github.com/open-pencil/open-pencil) (`docs/design/pirsig-mobile.fig`) screen for screen: Today, Notes, a
note, the slip-box, PARA, a project, More, and the capture sheet. The sections it does not have
yet (Hippos, feeds, Plasticity, Readiness, editing a note) open the full page, which works at
phone width too.

| From the design | In the crate |
|---|---|
| the `Pirsig` variables, light and dark (`design_to_tokens`) | `style/tokens.css`, generated; the app sets `data-pirsig="dark"` from the system theme |
| the frames' auto-layout: padding, gaps, sizes, radii | `style/app.css`, each rule named after its frame |
| the Lucide icons (`export_svg` at 24px) | `src/icons.rs`, generated |
| the frames (01 Today … 08 More) | `src/screens/`, one module per screen; `src/ui.rs` the shared pieces |

Fonts are Inter and JetBrains Mono, bundled (`fonts/`, SIL OFL), since the page loads nothing
from elsewhere. `scripts/build-mobile-ui.sh` (`just mobile-ui`) builds the crate into
`dist/mobile/`, which is not committed; Tauri runs it before every build, so the app embeds it.
`src-tauri/gen/android`'s `MainActivity.kt` keeps the page between the system bars, with the
design's colours behind them. To see the phone UI in a browser: `just mobile-ui`, `just dev`,
then open <http://localhost:5150/mobile/> at phone width.

## Where the brain lives

| Run as | Folder the app runs in | Brain on first launch |
|---|---|---|
| `just desktop` (a checkout) | `PIRSIG_HOME` when set, else the current folder when it holds Pirsig's `config/`, else (debug builds) the checkout it was built from | the checkout's, untouched |
| `Pirsig.app` (`just desktop-build`) | its data folder, `~/Library/Application Support/io.github.h0ffmann.pirsig` | `[apps.macos] seed`, default `"template"`: the fork state of `[template] repo` |
| Android (`just android`) | its data folder on the phone | `[apps.android] seed`, default `"empty"` |

An app with no checkout unpacks what it was built with into its data folder on every launch,
the config (`config/development.yaml`) and the page (`desktop/dist`), and writes
`pirsig.toml` there once; that copy is yours to edit (another `[template] repo`, `seed =
"empty"`, the window size). Seeding happens only when the brain is empty, needs one HTTPS
request (no `git`), and if it fails the app opens on an empty brain; `psg bootstrap repo` does
the same by hand. Any process on the machine can reach the API's port while the app runs, as
with `just dev`.

## Build

| | |
|---|---|
| Both | The [wasm-bindgen CLI](https://github.com/wasm-bindgen/wasm-bindgen) of the version `crates/pirsig-mobile` pins, for the phone UI: `cargo install wasm-bindgen-cli --version 0.2.129 --locked`. |
| macOS | `just desktop-build`: `Pirsig.app` and a `.dmg` in `src-tauri/target/release/bundle/`, unsigned (right-click → Open the first time). |
| Android | The [Android prerequisites](https://v2.tauri.app/start/prerequisites/#android): Android Studio's SDK and NDK, `JAVA_HOME` on JDK 17, `ANDROID_HOME` and `NDK_HOME`, and `rustup target add aarch64-linux-android`. Then `just android-dev` (a device or emulator) or `just android` (an APK, unsigned for release). The project Android Studio opens is `src-tauri/gen/android`. |

| Path | What |
|---|---|
| `src-tauri/src/lib.rs` | finds or unpacks the app's folder, seeds an empty brain, starts the API, then the window (`/mobile/` on Android); Android's entry point |
| `src-tauri/gen/android/…/MainActivity.kt` | edge to edge, with the page padded clear of the system bars and the keyboard |
| `src-tauri/src/main.rs` | the macOS entry point: calls `lib.rs` |
| `src-tauri/tauri.conf.json` | app id, CSP, bundles (`app`, `dmg`), Android's minimum SDK |
| `src-tauri/gen/android` | the Android project (Gradle), from `tauri android init` |
| `dist/index.html`, `dist/app.css` | the page and its styles (light and dark) |
| `dist/js/api.js`, `ui.js`, `md.js` | the API client, a small DOM and dialog toolkit, the Markdown renderer |
| `dist/js/views/*.js` | one module per section |
| `../crates/pirsig-mobile` | the phone UI (Leptos, WebAssembly), built into `dist/mobile/` |
| `e2e/brain.spec.mjs` | the end-to-end story, every section, on a fresh brain |

## Tests

`just desktop-e2e` runs `e2e/` with Playwright in WebKit (the engine of the macOS webview):
it serves the API and UI on a throwaway brain (`target/e2e-brain.sqlite`) and goes through
every section, creating, editing, archiving and deleting. CI (`.github/workflows/desktop.yml`)
runs it on every change to `desktop/` and checks that the app compiles on macOS. On Linux,
install `libwebkit2gtk-4.1-dev` and its friends first
([Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)).

## Next

1. Signing and a release workflow: a notarised `.dmg`, and a signed APK or AAB for the Play
   Store.
2. The rest of the phone UI in Rust: Hippos, feeds, Plasticity, Readiness and the note
   editor, each drawn in OpenPencil first.
