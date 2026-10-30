//! 01 Today: capture, the counts, the inbox to process, active projects.

use futures::try_join;
use leptos::prelude::*;

use crate::{
    api, dates, icons,
    model::{Area, Note, Project, Readiness, Stats},
    route::{Route, SlipTab},
    ui::{ctx, inbox_row, Card, Failed, Header, Ic, IconLink, Loading, Section},
};

#[derive(Clone)]
struct Data {
    stats: Stats,
    ready: Readiness,
    inbox: Vec<Note>,
    projects: Vec<Project>,
    areas: Vec<Area>,
}

async fn load() -> Result<Data, api::Error> {
    let (stats, ready, inbox, projects, areas) = try_join!(
        api::get::<Stats>("/api/slipbox/stats"),
        api::get::<Readiness>("/api/readiness"),
        api::get::<Vec<Note>>("/api/slipbox/inbox"),
        api::all::<Project>("/api/projects"),
        api::all::<Area>("/api/areas"),
    )?;
    Ok(Data {
        stats,
        ready,
        inbox,
        projects,
        areas,
    })
}

#[component]
pub fn Today() -> impl IntoView {
    let c = ctx();
    let data = LocalResource::new(move || {
        c.version.track();
        load()
    });
    view! {
        <Header title="Today" sub=Signal::stored(c.today.long())>
            <IconLink icon=icons::SEARCH label="Search notes" href=Route::Notes(None).href() />
        </Header>
        {move || match data.get() {
            None => view! { <Loading /> }.into_any(),
            Some(Err(e)) => view! { <Failed error=e /> }.into_any(),
            Some(Ok(d)) => body(d).into_any(),
        }}
    }
}

fn stat(label: &'static str, value: String, tone: &'static str, href: String) -> impl IntoView {
    view! {
        <a class="stat press" href=href>
            <span class="label">{label}</span>
            <span class=format!("value {tone}")>{value}</span>
        </a>
    }
}

fn body(d: Data) -> impl IntoView {
    let c = ctx();
    let mut active: Vec<Project> = d
        .projects
        .into_iter()
        .filter(|p| p.status() == "active" && p.archived_at.is_none())
        .collect();
    active.sort_by(|a, b| {
        a.due_on
            .as_deref()
            .unwrap_or("9999")
            .cmp(b.due_on.as_deref().unwrap_or("9999"))
    });
    let area = move |id: i64| {
        d.areas
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.name.clone())
            .unwrap_or_default()
    };
    let inbox_count = d.inbox.len();
    // the ones touched last first: what was on your mind
    let mut recent: Vec<&Note> = d.inbox.iter().collect();
    recent.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    let inbox = recent.into_iter().take(3).map(inbox_row).collect();
    let projects = active
        .iter()
        .take(5)
        .map(|p| {
            view! {
                <a class="row item press" href=Route::Project(p.id).href()>
                    <div class="main grow">
                        <span class="name">{p.name.clone()}</span>
                        <span class="sub">{area(p.area_id)}</span>
                    </div>
                    <span class="due-label">{p.due_on.as_deref().map(|d| dates::due(d, c.today)).unwrap_or_default()}</span>
                </a>
            }
            .into_any()
        })
        .collect();
    view! {
        <div class="scroll today">
            <button class="capture press" on:click=move |_| c.capture.set(true)>
                <Ic icon=icons::FEATHER size=18 />
                <span class="grow">"Capture a thought…"</span>
                <span class="send"><Ic icon=icons::ARROW_UP size=18 /></span>
            </button>
            <div class="stats">
                {stat("Notes", d.stats.notes.to_string(), "", Route::Notes(None).href())}
                {stat("Links", d.stats.links.to_string(), "", Route::SlipBox(SlipTab::Tree).href())}
                {stat("Inbox", d.stats.inbox.to_string(), "accent", Route::SlipBox(SlipTab::Inbox).href())}
            </div>
            <div class="stats">
                {stat("Orphans", d.stats.orphans.to_string(), "warn", Route::SlipBox(SlipTab::Orphans).href())}
                {stat("Readiness", format!("{}/{}", d.ready.ready, d.ready.total), "ok", "/#/ready".into())}
            </div>
            <Section title="Inbox · to process">
                {(inbox_count > 3).then(|| view! {
                    <a class="action" href=Route::SlipBox(SlipTab::Inbox).href()>{format!("All {inbox_count}")}</a>
                })}
            </Section>
            <Card rows=inbox empty="Inbox zero. Capture a thought above." />
            <Section title="Active projects" />
            <Card rows=projects empty="No active projects." />
        </div>
    }
}
