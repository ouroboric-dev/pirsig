# Template readiness

Is Pirsig ready to be copied with **Use this template**? This survey lists what a public GitHub
template needs, what is done, and the evidence. The items marked *(checked)* are tested by
`just template-check` (`scripts/template-check.sh`), which CI runs on every pull request. The
repository settings live on GitHub, not in a file, so they are checked by hand; each comes with
the `gh` command that sets it.

Surveyed on 2026-10-28, on `main` after #19.

## 1. Repository settings (on GitHub)

- [x] **Template repository** is on (`is_template: true`).
- [x] **Public**, with the license detected by GitHub: `MPL-2.0`.
- [ ] **Description** says what Pirsig is; today it is "agentic 🧠".
  `gh repo edit h0ffmann/pirsig --description "無 Pirsig: a second brain, PARA and a Zettelkasten slip-box, in Rust: CLI, REST API, desktop app"`
- [ ] **Topics**, so people find it; today there are none.
  `gh repo edit h0ffmann/pirsig --add-topic second-brain,zettelkasten,para-method,pkm,rust,loco-rs,tauri,obsidian,template`
- [x] **Issues** are on: the issue forms in `.github/ISSUE_TEMPLATE/` need them.
  `gh repo edit h0ffmann/pirsig --enable-issues`
- [ ] **Private vulnerability reporting** is on: `SECURITY.md` sends reports there; today it is off.
  `gh api -X PUT repos/h0ffmann/pirsig/private-vulnerability-reporting`
- [ ] **Dependabot alerts** are on (the version updates in `.github/dependabot.yml` work without them).
  `gh api -X PUT repos/h0ffmann/pirsig/vulnerability-alerts`
- [ ] **`main` is protected**: changes through pull requests, with CI green. Today there is no rule.
  Settings → Rules → Rulesets, or `gh api -X POST repos/h0ffmann/pirsig/rulesets` with a
  `pull_request` and a `required_status_checks` rule.
- [ ] **Head branches are deleted after a merge.**
  `gh repo edit h0ffmann/pirsig --delete-branch-on-merge`
- [ ] **Social preview**: upload [`docs/img/social-preview.png`](img/social-preview.png) (1280 × 640)
  in Settings → General → Social preview. GitHub has no API for it.
- [ ] **A first release**, `v0.1.0`, once this is merged: `CHANGELOG.md` is ready for it.
  `gh release create v0.1.0 --title "Pirsig 0.1.0" --notes-file CHANGELOG.md`
- [ ] *(optional)* Wiki and Projects off, if unused: `gh repo edit h0ffmann/pirsig --enable-wiki=false --enable-projects=false`.

## 2. Files a copy starts with

- [x] `LICENSE` is the MPL-2.0, and every crate and `package.json` says so. *(checked)*
- [x] `README.md` says how to use the template ("Use this template") and has a License section. *(checked)*
- [x] `CONTRIBUTING.md`: set up, make a change, commit messages, privacy. *(checked)*
- [x] `CODE_OF_CONDUCT.md`: the Contributor Covenant 2.1, its official text. *(checked)*
- [x] `SECURITY.md`: how to report privately, what is in scope, how to keep a copy safe. *(checked)*
- [x] `CHANGELOG.md` with `0.1.0`. *(checked)*
- [x] `.github/CODEOWNERS`, a pull request template, and issue forms (bug, idea; blank issues off,
  security reports sent to the private channel). *(checked)*
- [x] `.github/dependabot.yml`: Cargo (the workspace and the desktop app), npm (`desktop/`,
  `obsidian-painter/`) and GitHub Actions, weekly and grouped. *(checked)*
- [x] `.editorconfig`. *(checked)*

## 3. Nothing private ships

- [x] No brain database, `.env`, key or `config/*.local.yaml` is tracked. *(checked)*
- [x] No token or private key in any tracked file (GitHub, Anthropic and AWS patterns, PEM keys). *(checked)*
- [x] `config/production.yaml` takes every secret from the environment and has no default for
  `JWT_SECRET`. *(checked)*
- [x] The development `JWT_SECRET` can be overridden from the environment. *(checked)*
- [x] The project's Claude Code settings enable no personal plugin: the author's plugins moved to
  `.claude/settings.local.json`, which `.gitignore` now covers. *(checked)*
- [x] `brain/` holds no `#private` note and names no private repository. *(checked)*

## 4. A copy can become someone else's

- [x] `just adopt <owner> [<repo>]` (`scripts/adopt.sh`) points the copy at its new owner: the
  repository URL (README badges and clone lines, the plugin, the GitHub App manifest, the
  importers' user agent), the desktop app id, CODEOWNERS and the contacts in `SECURITY.md` and
  `CODE_OF_CONDUCT.md`. It runs on the bash 3.2 that macOS ships. *(checked: adopted as
  `acme/brain` in a copy, nothing of `h0ffmann/pirsig` is left outside `brain/`, `examples/` and
  `tests/`; the GitHub App manifest tests pass in the adopted copy)*
- [x] The README says what a copy does next: adopt, `just brain-reset` or `just fork-state`, and
  how to take later versions from `upstream`.
- [x] `pirsig.toml` says how a copy runs, without editing code: the repository a new brain is
  bootstrapped from, the fork state's folder, the defaults of `psg gh`, and what the apps start
  from. Every key is optional and an unknown one is an error; the shipped file is exactly the
  defaults (a unit test says so). `just adopt` points `[template] repo` at the copy. *(checked)*
- [x] A brain can start from any Pirsig repository: `psg bootstrap repo [OWNER/REPO | URL | DIR]`
  reads its fork state from one tarball, and an installed macOS app does that on first launch.

## 5. CI a copy inherits

- [x] Every action is pinned to a commit, as supply-chain hygiene. `ci.yaml` was not:
  `actions/checkout@v4`, `dtolnay/rust-toolchain@stable` and `Swatinem/rust-cache@v2` are now
  pinned, and the archived `actions-rs/cargo` is replaced by plain `cargo` steps. *(checked)*
- [x] Every workflow declares its `permissions` (read-only unless a job needs more). *(checked)*
- [x] CI needs no secret on a pull request; the plugin evals, which cost API credits, run only by
  hand.
- [x] A `template` job runs `scripts/template-check.sh`.

## 6. A fresh copy works

- [x] "Pirsig in a minute" was run, command by command, on a fresh brain.
- [x] `just check` (fmt, clippy pedantic, every test), `just desktop-e2e` (11 WebKit tests) and
  `just figure-check` pass.
- [x] The README logo and the figures need no network; `vendor/archify` is a submodule that only
  `just figure` needs (`just figure-init`).

## 7. Known gaps

- [ ] **No signed installers.** The app targets macOS and Android. `just desktop-build` makes an
  unsigned `Pirsig.app` and `.dmg`, and `just android` an APK, but there is no notarised build or
  Play Store bundle to download yet.
- [ ] **A copy has no shared history.** GitHub templates copy files, not commits, so the first
  `git pull upstream main` needs `--allow-unrelated-histories` (the README says so).
- [ ] **`brain/` is the author's.** A copy starts with h0ffmann's public notes, on purpose (the
  fork state); `just adopt` says how to drop them.
- [ ] **CI's test job starts Postgres and Redis** from the Loco starter, while Pirsig runs on
  SQLite. Worth a look on its own: the tests may lean on `DATABASE_URL`.
