# Contributing

Thanks for helping. Pirsig is a Rust app ([Loco](https://loco.rs)), with a desktop app (Tauri)
and plain HTML, CSS and JavaScript for its UI.

## Set up

1. Install [rustup](https://rustup.rs) and [`just`](https://github.com/casey/just).
   `rust-toolchain.toml` pins the Rust version; the first `cargo` command installs it.
   `just` puts rustup's `cargo` ahead of any other on PATH (a Homebrew `rust` ignores the pin).
2. `just check` runs what CI runs: formatting, clippy (pedantic) and every test.
3. `just dev` serves the API and the UI on http://localhost:5150; `just desktop` opens the app.
4. For the UI tests: `just desktop-e2e` (Node and Playwright's WebKit).

## Make a change

- One change per pull request, on a branch from `main`.
- Code reads like the code around it: same naming, same comment density.
- Every behaviour change comes with a test. Model rules hold in every interface, so test them
  through the API (`tests/requests/`), not only the CLI.
- Keep `just check` green. Run `just figure-check` if you touched a diagram, and
  `just template-check` if you touched a repository-wide file.
- Docs change with the code: the README, `docs/methodology.md` and the design notes in
  `docs/design/` say what is built.

## Commit messages

A short summary line in English, a body that says why, and a `Tested:` line that says how:

```
Add psg fork export: the brain as a public vault

...

Tested: just check; fork export on a copy of the brain
```

## Privacy

Never commit a brain database (`*.sqlite`), a token or a `config/*.local.yaml`. A change to
`psg fork export` must keep every `#private` note, and every mention of one, out of `brain/`.

## License

By contributing, you agree that your contribution is licensed under the
[Mozilla Public License 2.0](LICENSE), like the rest of Pirsig.
