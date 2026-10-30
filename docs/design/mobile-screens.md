# Mobile screens

A first pass at Pirsig on a phone: the desktop app's views, reshaped for one hand. The
sidebar becomes a tab bar (Today, Notes, Slip-box, PARA, More) and quick capture becomes a
sheet. Colours, kinds and words come from the desktop app (`desktop/dist/app.css` and the
views in `desktop/dist/js/views/`), so the two read as one app.

The screens were drawn in [OpenPencil](https://github.com/open-pencil/open-pencil) through
its MCP server, at 390×844 and exported at 2×. They are mock-ups with sample data; the
[app built from them](#the-app) follows.

| Today | Notes | Note |
|---|---|---|
| <img alt="Today: quick capture, counts for notes, links, inbox, orphans and readiness, the inbox to process and active projects" src="../img/mobile/today.png" width="260"/> | <img alt="Notes: search, chips to filter by kind, and notes with their ids, kinds and hashtags" src="../img/mobile/notes.png" width="260"/> | <img alt="A permanent note: its body with a wikilink to 1a, a quote, tags, then its links and backlinks" src="../img/mobile/note.png" width="260"/> |

| Slip-box | Project | Capture |
|---|---|---|
| <img alt="Slip-box: Inbox, Orphans, Tree and Tags; the tree shows notes in Folgezettel order, continuations indented under the note they continue" src="../img/mobile/slipbox.png" width="260"/> | <img alt="The project Run a 10k in the Health area: due date, task progress, tasks and the notes filed under it" src="../img/mobile/project.png" width="260"/> | <img alt="The capture sheet over Today: title, body, a kind picker and pickers for project and Hippo" src="../img/mobile/capture.png" width="260"/> |

## The app

`crates/pirsig-mobile` builds these screens in Rust (Leptos, compiled to WebAssembly), and
the Android app opens them at `/mobile/`. Its colours and icons are exported from the design
file, `docs/design/pirsig-mobile.fig`, and its sizes are read from the frames' auto-layout;
`desktop/README.md` says how. Below, each frame beside a screenshot of the app: Chromium at
390×844 and 2× (Android's WebView is Chromium), on a brain seeded with the design's notes,
projects and tasks.

| Screen | Design | App |
|---|---|---|
| Today | <img alt="Today as drawn" src="../img/mobile/today.png" width="260"/> | <img alt="The app's Today: the counts, the three notes touched last in the inbox, and active projects by due date" src="../img/mobile/app/today.png" width="260"/> |
| Notes | <img alt="Notes as drawn" src="../img/mobile/notes.png" width="260"/> | <img alt="The app's Notes: search, kind chips, and every note, newest first" src="../img/mobile/app/notes.png" width="260"/> |
| Note | <img alt="Note as drawn" src="../img/mobile/note.png" width="260"/> | <img alt="The app's note 1: its body with the wikilink to 1a, the quote and tags, then two links and one backlink" src="../img/mobile/app/note.png" width="260"/> |
| Slip-box | <img alt="Slip-box as drawn" src="../img/mobile/slipbox.png" width="260"/> | <img alt="The app's slip-box tree: notes in Folgezettel order, continuations indented" src="../img/mobile/app/slipbox.png" width="260"/> |
| Project | <img alt="Project as drawn" src="../img/mobile/project.png" width="260"/> | <img alt="The app's Run a 10k project: due in 171 days, 2 of 5 tasks done, and two notes with their tags" src="../img/mobile/app/project.png" width="260"/> |
| Capture | <img alt="Capture as drawn" src="../img/mobile/capture.png" width="260"/> | <img alt="The app's capture sheet over Today, filled in, with the project Run a 10k picked" src="../img/mobile/app/capture.png" width="260"/> |

Where the app differs:
- The counts are the seeded brain's (19 notes, not 128).
- Notes lists the newest first, as the API pages them; Today lists projects by due date.
- Rows are at least 44px tall, so a thumb can hit them.

In dark mode, the app follows the phone's theme:

| Today | Notes | Note |
|---|---|---|
| <img alt="Today in dark mode" src="../img/mobile/app/today-dark.png" width="260"/> | <img alt="Notes in dark mode" src="../img/mobile/app/notes-dark.png" width="260"/> | <img alt="Note 1 in dark mode" src="../img/mobile/app/note-dark.png" width="260"/> |

What the screens map to:

| Screen | Desktop view | API |
|---|---|---|
| Today | `#/` (`today.js`) | `/api/slipbox/stats`, `/api/readiness`, `/api/slipbox/inbox`, `/api/projects`, `/api/tasks` |
| Notes | `#/notes` | `/api/notes` |
| Note | `#/notes/:zid` | `/api/notes/:zid` |
| Slip-box | `#/slipbox/:tab` | `/api/slipbox/{inbox,orphans,tree}`, `/api/slipbox/stats` |
| Project | `#/projects/:id` | `/api/projects/:id`, `/api/projects/:id/notes`, `/api/tasks` |
| Capture | the sidebar's capture form | `POST /api/notes` |

Not drawn yet: Hippos, Areas, Resources, Feeds and arXiv, Plasticity, Readiness, the note
editor, and dark mode (the app has a dark theme, from the design's Dark variables).
