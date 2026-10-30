<div align="center">

<img alt="無 (mu, no thing), the Pirsig logo: the kanji brushed in its twelve strokes, drawn from Desmos equations, by @h0ffmann" src="docs/drawings/pirsig.svg" width="200"/>

# Pirsig

<sub>無 · <i>mu</i> · a second brain · <a href="#pirsig-in-a-minute">in a minute</a> · <a href="#ways-to-run-pirsig">run it</a> · <a href="docs/drawings/README.md">the logo, in Desmos</a> · <a href="#license">license</a></sub>

[![ci](https://github.com/h0ffmann/pirsig/actions/workflows/ci.yaml/badge.svg)](https://github.com/h0ffmann/pirsig/actions/workflows/ci.yaml)
[![license: MPL-2.0](https://img.shields.io/badge/license-MPL--2.0-1682D4?style=flat-square)](#license)
[![logo: Desmos](https://img.shields.io/badge/logo-Desmos-FF4F00?style=flat-square)](docs/drawings/pirsig.desmos.json)

</div>

A second brain from the future: PARA and a Zettelkasten slip-box in one place. Pirsig is a
[Loco](https://loco.rs) app in Rust. A desktop app, a REST API and a terminal CLI, `psg`,
share one model layer, so the rules are the same wherever you work.

> Pirsig is named for **Robert M. Pirsig** (1928–2017), the author of
> [*Zen and the Art of Motorcycle Maintenance*](https://en.wikipedia.org/wiki/Zen_and_the_Art_of_Motorcycle_Maintenance)
> (1974) and *Lila* (1991). He is my favorite author and a great inspiration to all of my
> work. On 24 April 2027 it will be ten years without him.
>
> The logo is 無, *mu*, "no thing": in chapter 26 of *Zen and the Art of Motorcycle
> Maintenance*, Pirsig takes it from Zen as the answer that is neither yes nor no, and writes
> that "science grows by its mu answers more than by its yes or no answer" (p. 413 in the
> edition [LitCharts](https://www.litcharts.com/lit/zen-and-the-art-of-motorcycle-maintenance/symbols/mu)
> cites; see [docs/drawings](docs/drawings/README.md)).

## Pirsig in a minute

You need [Rust](https://rustup.rs) (the repository pins the version) and
[`just`](https://github.com/casey/just).

```sh
git clone https://github.com/h0ffmann/pirsig.git && cd pirsig
just desktop                     # the desktop app; or `just dev` and open http://localhost:5150
```

Or stay in the terminal:

```sh
just install                                       # puts `psg` on your PATH
psg note -c "Atomic notes beat long documents"     # capture a thought (the inbox)
psg note -c -t "Links beat folders" -k permanent -b "Because [[1]] says so. #pkm"
psg area -c -n Health && psg project -c -n "Run a 10k" -a 1 --due 2027-04-18
psg ready                                          # what the brain still lacks, and how to fix it
```

Then pick a starting point: `just fork-state` loads my public notes, `just brain-reset`
fills the brain from your own GitHub (see [below](#start-from-my-brain-or-from-yours)).

## Ways to run Pirsig

| | How | Status |
|---|---|---|
| **Self-hosted, live** | The desktop app (Tauri, Rust): `just desktop`. Or the server alone, `just dev`, with the same UI in a browser. Your brain stays on your machine, in SQLite. | available |
| **Android app** | The same app on your phone (Tauri): `just android-dev` on a device or emulator, `just android` for an APK. The brain lives on the phone. | from source |
| **Mobile cloud** | Pirsig hosted for you at [piabrota.art/pirsig](https://piabrota.art/pirsig). | coming |

## How it works

![How Pirsig works: the psg CLI and the REST API with its desktop app make the same calls into one model layer, which reads and writes SQLite; psg figure exports the PARA tree for archify, which draws a checked HTML page and PNG; psg graph exports a graph.json for graphify, which clusters, queries and draws it.](docs/img/how-pirsig-works.png)

<details open>
<summary>How to read this figure</summary>

**Takeaway.** Whatever you use, the terminal, the desktop app or HTTP, goes through the same rules, and the brain can
leave Pirsig as a diagram or as a knowledge graph.

**How to read.** Top to bottom, one lane per role. Arrows say what flows. The orange border marks
where you come in and the outside tools that draw the result.

**Not shown.** Auth, the background workers, and the archive (`--archived`) and done tasks
(`--all`) switches of the exports.

**Evidence.** [`docs/archify/how-pirsig-works.json`](docs/archify/how-pirsig-works.json), the source of
the figure; `src/cli/mod.rs`, `src/controllers/`, `src/figures/`.

</details>

## Use this template

This repository is a GitHub template: **Use this template → Create a new repository**, and
make the new repository **private**, because your brain is personal data. Then:

```sh
git clone git@github.com:<you>/<brain>.git && cd <brain>
just adopt <you> <brain>   # its URLs, app id, owners and contacts become yours; review with git diff
just brain-reset           # your own GitHub in the brain (or: just fork-state, to start from mine)
git commit -am "Adopt Pirsig" && git push
```

Then make it work your way in [`pirsig.toml`](pirsig.toml): the repository a new brain is
bootstrapped from (`just adopt` points it at your copy), the fork state's folder, the defaults
of `psg gh`, and what the macOS and Android apps start from on first launch. Every key is
optional, and a key Pirsig does not know is an error, so a typo shows. The Loco side (port,
database, logging) stays in `config/`.

To take later versions of Pirsig, add this repository as `upstream`:
`git remote add upstream https://github.com/h0ffmann/pirsig.git`, then
`git pull upstream main --allow-unrelated-histories` the first time and `git pull upstream main`
after that. `just template-check` says whether a copy is still fit to be a template itself;
[docs/template-readiness.md](docs/template-readiness.md) is the full survey.

## Start from my brain, or from yours

This repository carries my brain's public notes in [`brain/`](brain/), a Markdown vault (the
Obsidian export format) that reads well on GitHub too:

- **Featured repositories**, my own repositories pushed in the last two years: a note and a
  resource each, indexed by category in hub notes under the areas I keep.
- **Public forks**, one index note, a line per fork, by category.
- **Cold repositories**, the own ones with no push for two years, in one more index note.

```sh
just fork-state        # start from brain/: my note structure, in your brain
just brain-reset       # or replace the whole brain with your own GitHub (psg gh --overwrite)
just fork-export       # write your brain to brain/, for whoever copies yours
psg bootstrap repo acme/brain   # or start from any Pirsig repository's brain/, no clone needed
```

`psg bootstrap repo` takes `owner/repo`, a GitHub URL or a local checkout, and defaults to
`[template] repo` in `pirsig.toml`; it downloads one tarball and reads only its fork state. The
macOS app does the same on first launch, so an installed app starts from the template's brain.

`psg gh` takes the same depth controls: `--cold-after MONTHS` (default 24) and
`--forks index|notes|skip`, with their defaults in `pirsig.toml`'s `[github]`. Both `fork-state` and `brain-reset` back the brain up next to its
database file before they replace it. `psg fork export` leaves out every note tagged
`#private`, the resources only those notes cite, and any link or list line that would name
them, so private repositories never reach `brain/`.

## Your brain is private

Your brain is personal data; this repository is public code. Keep your own brain in a
**private copy** of this repository, so nothing you write ends up on the web because it is
in git:

```sh
# create an empty private repository first, e.g. github.com/<you>/brain
git clone https://github.com/h0ffmann/pirsig.git brain && cd brain
git remote rename origin upstream
git remote add origin git@github.com:<you>/brain.git
git push -u origin main
git pull upstream main     # later, to update Pirsig
```

Publishing will be a separate, explicit step (designed, not built): `psg publish` shows a
plan of exactly what would become public, like `terraform plan`, and only
`psg publish --apply` does it. See [docs/design](docs/design/README.md).

## What is in the brain

### PARA

| | What | API | CLI |
|---|---|---|---|
| **P**rojects | outcomes with an end, inside an area | `/api/projects` | `psg project` |
| **A**reas | ongoing responsibilities | `/api/areas` | `psg area` |
| **R**esources | reference material, optionally tied to an area or project | `/api/resources` | `psg resource` |
| **A**rchives | `archived_at` on areas, projects, resources | `POST /api/<kind>/{id}/archive`, `?archived=true` | `--archive ID`, `--archived` |

Tasks (`/api/tasks`, `psg task`) are next actions inside a project. The same rules hold in
every interface:

- Names are never blank, and area names are unique. A project's `status` is `active` (the
  default), `waiting`, `someday` or `done`; `--due` gives it its end date.
- Nothing active lives in the archive. Archiving an area archives its projects and
  resources, and nothing new goes into an archived area or project.
- Deleting an area or project that still has children needs `--force` (`?force=true`).
- A resource can carry a `kind` (`paper`, `book`, `repo`, ...) and a unique `citekey` that
  literature notes cite.
- A project pulls notes in by link, without owning them:
  `psg project --pull 2 --note 1a -w "the training plan"`.

### Slip-box

Notes have Luhmann-style ids (`zid`): `1` starts a train of thought, `1a` continues it,
`1a1` branches off `1a`, `2` starts the next one.

- **Kinds**, after *How to Take Smart Notes*: `fleeting` (the inbox), `literature` (cites a
  `source` or the resource it was written from), `permanent` (your own words) and `hub`
  (structure).
- **Links**: `[[1a]]` or `[[Some title]]` in a body becomes a link on save, even when the
  target comes later. Renaming a note rewrites the links to it. Hand-made links
  (`psg link A B -w "why"`) carry a reason.
- **Labels**: `#hashtags` in the body plus `--tag`
  ([labels and tags](docs/design/labels-and-tags.md)).
- **Views**: backlinks, inbox, orphans, random, the Folgezettel tree, the graph, stats.

```sh
psg note -c -t "Links beat folders" -a 1 -k permanent -e   # continue 1 as 1a, in $EDITOR
psg note -s 1a                                  # the note, its links and backlinks
psg note rust                                   # search; also -k KIND, --tag TAG
psg link 1a 2 -w "contradicts"
psg box tree                                    # also inbox, orphans, random, graph, stats
```

### Hippo: a rhizome of notes

A **Hippo** is a way of thinking: a name, a description and *fixations*, the subjects its
notes gather around. Its notes form a **rhizome**, not a tree: each note is linked in when it
is saved (to notes that share a fixation, a tag or a title word, else to the last one
captured, and every link says why), `--after` is refused, and deleting a note links its
neighbours to each other.

```sh
psg hippo -c -n h0 --fixation "rogue waves" --fixation chess
psg note -c "the Draupner wave was linear focusing?" -H 1   # woven in on save
psg hippo --map 1      # where you left off, each fixation's notes, any pieces
psg hippo --weave 1    # join the pieces
```

### Plasticity: quick exercises

A **plast** is a kind of quick exercise that trains the brain itself: six ship, one per
domain (math, abstraction, logic, music, literature, languages), five levels each. Right
answers push a plast out and three in a row raise its level; wrong ones bring it back.

```sh
psg plast go            # five exercises, the most due first (or: just plast)
psg plast go -d music   # one domain
```

### In and out

| Channel | CLI | API |
|---|---|---|
| GitHub repos in: featured notes, fork and cold indexes | `psg gh [--private minimal] [--readme] [--cold-after 24] [--forks index] [--overwrite]` | |
| The fork state out and in | `psg fork export`, `psg fork import [--overwrite]` | |
| Obsidian vault out and in | `psg obsidian export ./vault`, `psg obsidian import ./vault [--dry-run]` | |
| RSS/Atom feeds in, as resources | `psg rss add URL -a AREA`, `psg rss pull` | `/api/feeds` |
| The slip-box out, as RSS | `psg rss out -o feed.xml` | `GET /api/slipbox/feed.xml` |
| arXiv papers in: cited, filed, deduplicated | `psg arxiv add ID... [--note]`, `psg arxiv search QUERY` | `POST /api/ingest/arxiv` |
| Methodology readiness | `psg ready` | `GET /api/readiness` |

`psg ready` checks the brain against PARA, the Zettelkasten method and the channels, and says
what to run for each gap. [docs/methodology.md](docs/methodology.md) has the readiness
matrix: what is enforced, what is only reported, and where each rule is tested.

## Apps and tools

### Desktop and Android apps

The app ([`desktop/`](desktop/README.md), Tauri 2) targets macOS and Android. `just desktop`
opens it on the same brain as `psg`; `just desktop-build` makes a `Pirsig.app`, which keeps its
brain in its own data folder and, on first launch, bootstraps it from the template's fork
state. `just android-dev` and `just android` run it on a phone. Everything is managed there: PARA and the archive, notes with their links and
backlinks, the slip-box views, Hippos, feeds and arXiv, Plasticity and readiness. The app runs
the REST API inside itself and the API serves the UI, so `just dev` shows the same UI in a
browser. `just desktop-e2e` drives it end to end in WebKit.

### Claude Code plugin

The `pirsig` plugin brings three skills to Claude Code: `pirsig` (work in the brain),
`zk-method` (check notes against the method) and `hippo` (grow a Hippo's rhizome).

```sh
claude plugin marketplace add h0ffmann/pirsig
claude plugin install pirsig@pirsig
```

`scripts/plugin-smoke.sh` installs it from a checkout into an empty config, as a third party
would; CI runs it on every change. The evals in `plugins/pirsig/evals/` run by hand
(Actions → plugin), because they cost API credits.

### obsidian-painter

An Obsidian plugin in [`obsidian-painter/`](obsidian-painter/README.md), mostly Rust compiled
to WebAssembly: ```` ```desmos ```` blocks draw offline, ```` ```archify ```` blocks show
figures, and "Export note to PDF" writes a paper-quality PDF through pandoc. `just painter`
builds and tests it.

### Figures and graphs

`psg figure` exports the PARA tree as [archify](https://github.com/tt-a1i/archify) JSON, and
`psg graph` the brain as a [graphify](https://github.com/safishamsi/graphify) `graph.json`
(also `GET /api/figures/tree` and `/graph`).

```sh
psg figure -o docs/archify/para.json && just figure     # -> docs/img/para.png
just graph                                              # -> graphify-out/graph.html
```

![The demo brain as a tree: the root "A second brain in Pirsig" has the areas Health, Work and Second brain and an Unfiled resources node; Health holds the project Run a marathon with its tasks and the resource Couch to 5k; Work holds Q4 roadmap; Second brain holds Ship Pirsig v0 with two tasks and the resource PARA method; Reading list is unfiled.](docs/img/para.png)

<details open>
<summary>How to read this figure</summary>

**Takeaway.** This is `psg para` as a picture: every area, project, open task and resource in its
place.

**How to read.** Top down, each line means "contains". Below the root are the areas, then their
projects, then tasks and resources. Small grey text is the description, status, due date or URL.

**Not shown.** Done tasks and the archive; `psg figure --all --archived` puts them in.

**Evidence.** [`docs/archify/para.json`](docs/archify/para.json), written by `psg figure` from the
demo brain of `just figure-para`; the drawing is archify's.

</details>

Diagrams follow the `figure` skill: sources in `docs/archify/`, `just figure` draws them with
the vendored archify, and `just figure-check` says whether a PNG is stale.

## Reference

- **CLI.** `just install` puts `psg` on your PATH (or `cargo run --bin psg -- <cmd>`). Every
  command takes `--json`. `psg` reads the config of `LOCO_ENV` (default `development`) from
  `./config`, and `./pirsig.toml` (or the file `PIRSIG_CONFIG` names); `DATABASE_URL` picks
  another brain. Pending migrations run on their own.
- **justfile.** `just n "An idea"`, `just show 1a`, `just find rust`, `just ready`,
  `just dev`, `just check` (fmt, clippy and tests, as CI). `just` lists them all.
- **Server.** `cargo loco start` serves the API and the UI on http://localhost:5150. Rust 1.94
  or newer; `rust-toolchain.toml` pins it, so rustup installs it on the first `cargo` command.
- **Drawing.** The logo at the top is 無 drawn from its Desmos expressions by
  `cargo run -p pirsig-drawings --bin pirsig-art`; see [docs/drawings](docs/drawings/README.md).
- **Use cases.** [docs/use-cases.md](docs/use-cases.md) builds a brain out of one person's
  GitHub repositories, with a sample slip-box in [`examples/slipbox/`](examples/slipbox/).

## License

Pirsig is licensed under the [Mozilla Public License 2.0](LICENSE) (MPL-2.0): use it, change it and
ship it, and share your changes to its files under the same license.
