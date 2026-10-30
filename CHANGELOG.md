# Changelog

What changes in Pirsig, newest first. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- `pirsig.toml`: how a copy runs — the template repository a brain is bootstrapped from, the
  fork state's folder, the defaults of `psg gh`, and what the apps start from. Every key is
  optional; unknown keys are an error.
- `psg bootstrap repo [OWNER/REPO | URL | DIR]`: start a brain from any Pirsig repository's fork
  state, from one tarball (no `git`), with `--overwrite` to replace a brain after a backup.
- The app targets macOS and Android. An installed `Pirsig.app` (`just desktop-build`) keeps its
  brain in its data folder and bootstraps it from the template on first launch; Android builds
  with `just android` / `just android-dev`.

### Changed

- `psg fork export|import` default to `[brain] fork_state`, and `psg gh` takes its defaults
  from `[github]`.

## [0.1.0] - unreleased

The first version meant to be copied, as a GitHub template.

### Added

- PARA (areas, projects, tasks, resources, the archive) and a Zettelkasten slip-box (zids,
  kinds, links with reasons, backlinks, the Folgezettel tree), the same through the `psg` CLI,
  the REST API and the desktop app.
- The desktop app (Tauri): the whole brain from one window, with the REST API inside it.
- Hippos (rhizomes of notes grown around fixations) and Plasticity (quick exercises).
- Channels: GitHub (`psg gh`, with depth control), arXiv, RSS in and out, Obsidian in and out.
- The fork state: `brain/`, `psg fork export|import`, `psg gh --overwrite`.
- `psg ready`: the brain checked against its methods.
- Template support: `just adopt`, `just template-check` and the readiness survey in
  `docs/template-readiness.md`.

### Changed

- The app is Pirsig and its CLI `psg` (formerly `ztk`); old vaults with `ztk:` keys still import.
- Licensed under the Mozilla Public License 2.0.
