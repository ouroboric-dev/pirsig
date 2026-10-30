//! 02 Notes: search, the kinds as chips, every note. 03 Note: one note, its
//! body, links and backlinks.

use leptos::prelude::*;

use crate::{
    api, dates, icons,
    model::{Note, NoteDetail, Page},
    route::{encode, Route},
    ui::{
        body, ctx, link_row, note_row, Card, Failed, Header, Ic, IconButton, IconLink, Loading,
        Nav, Pill, Section,
    },
};

pub const KINDS: [&str; 4] = ["fleeting", "literature", "permanent", "hub"];

#[component]
pub fn Notes(kind: Option<String>) -> impl IntoView {
    let c = ctx();
    let q = RwSignal::new(String::new());
    let filter = kind.clone();
    let page = LocalResource::new(move || {
        c.version.track();
        let mut path = format!("/api/notes?page=1&page_size=50&q={}", encode(&q.get()));
        if let Some(k) = &filter {
            path = format!("{path}&kind={}", encode(k));
        }
        async move { api::get::<Page<Note>>(&path).await }
    });
    let filtered = kind.is_some();
    let sub = Signal::derive(move || match page.get() {
        Some(Ok(p)) if q.with(String::is_empty) && !filtered => {
            format!("{} notes in the slip-box", p.total_items)
        }
        Some(Ok(p)) => format!("{} notes match", p.total_items),
        _ => String::new(),
    });
    let chips = std::iter::once(("All".to_string(), None))
        .chain(
            KINDS
                .iter()
                .map(|k| ((*k).to_string(), Some((*k).to_string()))),
        )
        .map(|(label, k)| {
            let on = k == kind;
            view! { <a class="chip press" class:on=on href=Route::Notes(k).href()>{label}</a> }
        })
        .collect_view();
    view! {
        <Header title="Notes" sub=sub accent=true>
            <IconButton icon=icons::PLUS label="New note" size=26 on=Callback::new(move |()| c.capture.set(true)) />
        </Header>
        <div class="scroll">
            <label class="search">
                <Ic icon=icons::SEARCH size=18 />
                <input
                    type="search"
                    placeholder="Search titles and bodies"
                    aria-label="Search notes"
                    prop:value=move || q.get()
                    on:input:target=move |ev| q.set(ev.target().value())
                />
            </label>
            <nav class="chips" aria-label="Kinds">{chips}</nav>
            {move || match page.get() {
                None => view! { <div class="skeleton"></div> }.into_any(),
                Some(Err(e)) => view! { <p class="error">{e}</p> }.into_any(),
                Some(Ok(p)) => {
                    let rows = p.items.iter().map(|n| note_row(n, None)).collect();
                    view! { <Card rows=rows empty="No notes match." /> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
pub fn NoteScreen(zid: String) -> impl IntoView {
    let c = ctx();
    let path = format!("/api/notes/{}", encode(&zid));
    let detail = LocalResource::new(move || {
        c.version.track();
        let path = path.clone();
        async move { api::get::<NoteDetail>(&path).await }
    });
    let desktop = format!("/#/notes/{}", encode(&zid));
    view! {
        <Nav back="Notes" href=Route::Notes(None).href()>
            <IconLink icon=icons::LINK_2 label="Link to a note" href=desktop.clone() />
            <IconLink icon=icons::PENCIL label="Edit" href=format!("/#/notes/{}/edit", encode(&zid)) />
            <IconLink icon=icons::ELLIPSIS label="More" href=desktop.clone() />
        </Nav>
        {move || match detail.get() {
            None => view! { <Loading /> }.into_any(),
            Some(Err(e)) => view! { <Failed error=e /> }.into_any(),
            Some(Ok(d)) => note(&d).into_any(),
        }}
    }
}

fn note(d: &NoteDetail) -> impl IntoView {
    let c = ctx();
    let n = &d.note;
    let links: Vec<AnyView> = d.links.iter().map(|l| link_row(&l.zid, &l.title)).collect();
    let backlinks: Vec<AnyView> = d
        .backlinks
        .iter()
        .map(|l| link_row(&l.zid, &l.title))
        .collect();
    // tags the body writes show where it writes them; the rest go under it
    let written = n.body.to_lowercase();
    let extra: Vec<String> = n
        .tags
        .iter()
        .filter(|t| !written.contains(&format!("#{t}")))
        .cloned()
        .collect();
    let has_extra = !extra.is_empty();
    let tags = extra
        .into_iter()
        .map(|t| view! { <span class="tag">{format!("#{t}")}</span> })
        .collect_view();
    view! {
        <div class="scroll wide">
            <div class="note-meta">
                <span class="mono">{n.zid.clone()}</span>
                <Pill kind=n.kind.clone() />
                <span>{format!("· edited {}", dates::ago(&n.updated_at, c.now, c.today))}</span>
            </div>
            <h1 class="note-title">{n.title.clone()}</h1>
            {body(&n.body, &d.links)}
            {has_extra.then(|| view! { <div class="tagline">{tags}</div> })}
            <Section title=format!("Links · {}", links.len()) />
            <Card rows=links empty="No links yet." />
            <Section title=format!("Backlinks · {}", backlinks.len()) />
            <Card rows=backlinks empty="Nothing links here yet." />
        </div>
    }
}
