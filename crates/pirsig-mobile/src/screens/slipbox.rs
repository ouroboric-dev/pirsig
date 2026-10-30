//! 04 Slip-box: the inbox, orphans, the Folgezettel tree and the tags.

use leptos::prelude::*;

use crate::{
    api, icons,
    model::{Note, NoteDetail, Stats, TreeEntry},
    route::{encode, Route, SlipTab},
    ui::{ctx, note_row, segments, Card, Failed, Header, IconButton, Loading, Pill},
};

#[derive(Clone)]
enum Data {
    Notes(Vec<Note>),
    Tree(Vec<TreeEntry>),
    Tags(Stats),
}

async fn load(tab: SlipTab) -> Result<(Stats, Data), api::Error> {
    let stats = api::get::<Stats>("/api/slipbox/stats").await?;
    let data = match tab {
        SlipTab::Inbox => Data::Notes(api::get("/api/slipbox/inbox").await?),
        SlipTab::Orphans => Data::Notes(api::get("/api/slipbox/orphans").await?),
        SlipTab::Tree => Data::Tree(api::get("/api/slipbox/tree").await?),
        SlipTab::Tags => Data::Tags(stats.clone()),
    };
    Ok((stats, data))
}

#[component]
pub fn SlipBox(tab: SlipTab) -> impl IntoView {
    let c = ctx();
    let data = LocalResource::new(move || {
        c.version.track();
        load(tab)
    });
    let random = Callback::new(move |()| {
        leptos::task::spawn_local(async move {
            match api::get::<NoteDetail>("/api/slipbox/random").await {
                Ok(d) => {
                    let _ = window()
                        .location()
                        .set_hash(&Route::Note(d.note.zid).href());
                }
                Err(e) => c.say(e, true),
            }
        });
    });
    view! {
        <Header title="Slip-box" accent=true>
            <IconButton icon=icons::SHUFFLE label="A random note" on=random />
            <IconButton icon=icons::PLUS label="New note" size=26 on=Callback::new(move |()| c.capture.set(true)) />
        </Header>
        {move || match data.get() {
            None => view! { <Loading /> }.into_any(),
            Some(Err(e)) => view! { <Failed error=e /> }.into_any(),
            Some(Ok((stats, d))) => body(tab, &stats, d).into_any(),
        }}
    }
}

fn body(tab: SlipTab, stats: &Stats, data: Data) -> impl IntoView {
    let tabs = [
        (SlipTab::Inbox, format!("Inbox {}", stats.inbox)),
        (SlipTab::Orphans, format!("Orphans {}", stats.orphans)),
        (SlipTab::Tree, "Tree".to_string()),
        (SlipTab::Tags, "Tags".to_string()),
    ]
    .map(|(t, label)| (t, label, Route::SlipBox(t).href()));
    let (intro, content) = match data {
        Data::Notes(notes) => {
            let rows = notes.iter().map(|n| note_row(n, None)).collect();
            let (intro, empty) = if tab == SlipTab::Inbox {
                (
                    "Fleeting notes: rewrite each as a permanent note in your words, or delete it.",
                    "Inbox zero.",
                )
            } else {
                (
                    "Permanent notes with no links in or out: link each to where it belongs.",
                    "No orphans: every permanent note is linked.",
                )
            };
            (intro, view! { <Card rows=rows empty=empty /> }.into_any())
        }
        Data::Tree(entries) => {
            // fleeting notes wait in the Inbox tab; the tree is the slip-box proper
            let rows = entries
                .iter()
                .filter(|n| n.kind != "fleeting")
                .map(tree_row)
                .collect();
            (
                "Every note in Folgezettel order: a continuation sits under the note it continues.",
                view! { <Card rows=rows empty="No notes yet." /> }.into_any(),
            )
        }
        Data::Tags(counts) => {
            let cloud = counts
                .tags
                .iter()
                .map(|t| {
                    view! {
                        <a class="tag" href=format!("/#/notes?tag={}", encode(&t.name))>
                            {format!("#{}", t.name)}
                            <span class="muted">{format!(" {}", t.count)}</span>
                        </a>
                    }
                })
                .collect_view();
            (
                "Every tag, with how many notes carry it.",
                view! { <div class="card tagcloud">{cloud}</div> }.into_any(),
            )
        }
    };
    view! {
        <div class="scroll">
            {segments(&tabs, tab)}
            <p class="intro">{intro}</p>
            {content}
        </div>
    }
}

fn tree_row(n: &TreeEntry) -> AnyView {
    let guides = (0..n.depth.min(8))
        .map(|_| view! { <span class="guide"></span> })
        .collect_view();
    view! {
        <a class="row tree press" href=Route::Note(n.zid.clone()).href()>
            {guides}
            <span class="zid mono">{n.zid.clone()}</span>
            <span class="title">{n.title.clone()}</span>
            {(n.kind != "permanent").then(|| view! { <Pill kind=n.kind.clone() /> })}
        </a>
    }
    .into_any()
}
