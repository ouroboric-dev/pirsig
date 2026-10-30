# Design notes

How Pirsig is meant to grow. Each note is a proposal: it says what we build, why, and what
stays out. Code that implements a note links back to it.

| Note | Channel | Summary |
|---|---|---|
| [Private by default: private fork + `psg publish`](private-fork-and-publish.md) | outbound | Keep the brain in a private copy of this repo; make things public only through a terraform-style plan/apply. |
| [Pandoc: papers and citations](pandoc.md) | outbound | Turn notes and resources into paper-quality documents with real citations, without the draft ever leaving your machine. |
| [`pirsig.dev.git`: git as an inbound channel](git-inbound.md) | inbound | Pull repositories and their READMEs into the brain as resources and atomic notes, linked to each other. |
| [`psg bootstrap gh`: your GitHub FOSS work](github-bootstrap.md) | inbound | Sign in with the Pirsig GitHub App and index your repos, public forks and org contributions (private ones metadata-only) as the brain's first layer. |
| [Labels and tags](labels-and-tags.md) | model | Labels are values (`#running`), tags are key/value pairs (`status=active`); how each maps to Obsidian, the CLI and the API. |
| [Mobile screens](mobile-screens.md) | app | Mock-ups of the phone app, drawn in OpenPencil: Today, Notes, a note, the slip-box tree, a project and the capture sheet. |

Built: `pirsig.pkm.obsidian` (vault out and in), `pirsig.web.rss` (feeds in, the slipbox out),
`pirsig.sci.arxiv` (papers in) and `psg bootstrap gh` (repos in, with a token; the GitHub
App sign-in is not built yet). [methodology.md](../methodology.md) says what each one
guarantees and how `psg ready` checks it.

**Inbound** channels bring things into the brain (git, web clipper, Obsidian import).
**Outbound** channels take things out of it (publish, pandoc, Obsidian export). Inbound
never needs permission: it only adds private material. Outbound always goes through an
explicit, reviewable step, because that is where personal data can leak.

Integrations are named `pirsig.<domain>.<tool>` (`pirsig.dev.git`, `pirsig.pub.pandoc`,
`pirsig.pkm.obsidian`), so the CLI, config keys and docs share one namespace.
