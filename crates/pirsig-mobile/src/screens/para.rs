//! 07 PARA: projects by status, areas, resources, open tasks. 05 Project: one
//! project, its progress, tasks and the notes it pulls in.

use futures::try_join;
use leptos::prelude::*;

use crate::{
    api, dates, icons,
    model::{Area, Project, ProjectNote, Resource, Task, TaskDone},
    route::{ParaTab, Route},
    ui::{ctx, segments, Card, Chevron, Failed, Header, Ic, IconLink, Loading, Nav, Pill, Section},
};

#[derive(Clone)]
enum Data {
    Projects(Vec<Project>, Vec<Area>),
    Areas(Vec<Area>),
    Resources(Vec<Resource>, Vec<Area>),
    Tasks(Vec<Task>, Vec<Project>),
}

async fn load(tab: ParaTab) -> Result<Data, api::Error> {
    Ok(match tab {
        ParaTab::Projects => {
            let (p, a) = try_join!(api::all("/api/projects"), api::all("/api/areas"))?;
            Data::Projects(p, a)
        }
        ParaTab::Areas => Data::Areas(api::all("/api/areas").await?),
        ParaTab::Resources => {
            let (r, a) = try_join!(api::all("/api/resources"), api::all("/api/areas"))?;
            Data::Resources(r, a)
        }
        ParaTab::Tasks => {
            let (t, p) = try_join!(api::all("/api/tasks?done=false"), api::all("/api/projects"))?;
            Data::Tasks(t, p)
        }
    })
}

fn name_of(areas: &[Area], id: i64) -> String {
    areas
        .iter()
        .find(|a| a.id == id)
        .map(|a| a.name.clone())
        .unwrap_or_default()
}

#[component]
pub fn Para(tab: ParaTab) -> impl IntoView {
    let c = ctx();
    let data = LocalResource::new(move || {
        c.version.track();
        load(tab)
    });
    let tabs = ParaTab::ALL.map(|t| (t, t.label().to_string(), Route::Para(t).href()));
    let add = format!("/#/{}", tab.slug());
    view! {
        <Header title="PARA" sub=Signal::stored("Projects, areas, resources, and the archive".to_string()) accent=true>
            <IconLink icon=icons::PLUS label=format!("New in {}", tab.label()) size=26 href=add />
        </Header>
        <div class="scroll">
            {segments(&tabs, tab)}
            {move || match data.get() {
                None => view! { <div class="skeleton"></div> }.into_any(),
                Some(Err(e)) => view! { <p class="error">{e}</p> }.into_any(),
                Some(Ok(d)) => body(d).into_any(),
            }}
        </div>
    }
}

fn body(d: Data) -> impl IntoView {
    match d {
        Data::Projects(projects, areas) => {
            let live: Vec<&Project> = projects
                .iter()
                .filter(|p| p.archived_at.is_none())
                .collect();
            let group = |statuses: &[&str]| -> Vec<AnyView> {
                live.iter()
                    .filter(|p| statuses.contains(&p.status()))
                    .map(|p| project_row(p, &areas))
                    .collect()
            };
            let active = group(&["active"]);
            let later = group(&["waiting", "someday"]);
            let done = group(&["done"]);
            view! {
                <Section title=format!("Active · {}", active.len()) />
                <Card rows=active empty="No active projects." />
                <Section title=format!("Waiting and someday · {}", later.len()) />
                <Card rows=later empty="Nothing waiting." />
                {(!done.is_empty()).then(|| view! {
                    <Section title=format!("Done · {}", done.len()) />
                    <Card rows=done />
                })}
            }
            .into_any()
        }
        Data::Areas(areas) => {
            let rows = areas
                .iter()
                .map(|a| {
                    view! {
                        <a class="row item press" href=format!("/#/areas/{}", a.id)>
                            <div class="main grow">
                                <span class="name">{a.name.clone()}</span>
                                {a.description.clone().map(|d| view! { <span class="sub">{d}</span> })}
                            </div>
                            <Chevron />
                        </a>
                    }
                    .into_any()
                })
                .collect();
            view! { <Section title=format!("Areas · {}", areas.len()) /><Card rows=rows empty="No areas yet." /> }.into_any()
        }
        Data::Resources(resources, areas) => {
            let rows = resources
                .iter()
                .map(|r| {
                    let sub = [r.kind.clone(), r.area_id.map(|id| name_of(&areas, id))]
                        .into_iter()
                        .flatten()
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                        .join(" · ");
                    view! {
                        <a class="row item press" href=format!("/#/resources/{}", r.id)>
                            <div class="main grow">
                                <span class="name">{r.title.clone()}</span>
                                <span class="sub">{sub}</span>
                            </div>
                            <Chevron />
                        </a>
                    }
                    .into_any()
                })
                .collect();
            view! { <Section title=format!("Resources · {}", resources.len()) /><Card rows=rows empty="No resources yet." /> }
                .into_any()
        }
        Data::Tasks(mut tasks, projects) => {
            tasks.sort_by(|a, b| {
                a.due_on
                    .as_deref()
                    .unwrap_or("9999")
                    .cmp(b.due_on.as_deref().unwrap_or("9999"))
            });
            let rows = tasks
                .iter()
                .map(|t| {
                    let project = projects
                        .iter()
                        .find(|p| p.id == t.project_id)
                        .map(|p| p.name.clone());
                    task_row(t, project)
                })
                .collect();
            view! { <Section title=format!("Open · {}", tasks.len()) /><Card rows=rows empty="Nothing to do." /> }.into_any()
        }
    }
}

fn project_row(p: &Project, areas: &[Area]) -> AnyView {
    let c = ctx();
    let status = p.status().to_string();
    view! {
        <a class="row item press" href=Route::Project(p.id).href()>
            <div class="main grow">
                <span class="name">{p.name.clone()}</span>
                <span class="sub">
                    {name_of(areas, p.area_id)}
                    <span>"·"</span>
                    <span class=format!("status-{status}")>{status.clone()}</span>
                </span>
            </div>
            {p.due_on.as_deref().map(|d| view! { <span class="due-label">{dates::due(d, c.today)}</span> })}
            <Chevron />
        </a>
    }
    .into_any()
}

/// A task with its box; ticking it tells the API and reloads.
pub fn task_row(t: &Task, project: Option<String>) -> AnyView {
    let c = ctx();
    let (id, done) = (t.id, t.done);
    let tick = move |_| {
        leptos::task::spawn_local(async move {
            match api::patch::<_, serde_json::Value>(
                &format!("/api/tasks/{id}"),
                &TaskDone { done: !done },
            )
            .await
            {
                Ok(_) => c.changed(),
                Err(e) => c.say(e, true),
            }
        });
    };
    let due = t.due_on.as_deref().map(|d| {
        let label = dates::due_short(d, c.today);
        let late = dates::days_until(d, c.today).is_some_and(|n| n <= 0) && !done;
        view! { <span class="due-label" class:today=late>{label}</span> }
    });
    view! {
        <div class="row task" class:done=done>
            <button
                class="box press"
                class:done=done
                role="checkbox"
                aria-checked=done.to_string()
                aria-label=format!("Done: {}", t.title)
                on:click=tick
            >
                {done.then(|| view! { <Ic icon=icons::CHECK size=15 /> })}
            </button>
            <div class="grow item">
                <div class="main">
                    <span class="title">{t.title.clone()}</span>
                    {project.map(|p| view! { <span class="sub">{p}</span> })}
                </div>
            </div>
            {due}
        </div>
    }
    .into_any()
}

#[derive(Clone)]
struct ProjectData {
    project: Project,
    area: Option<Area>,
    tasks: Vec<Task>,
    notes: Vec<ProjectNote>,
}

async fn load_project(id: i64) -> Result<ProjectData, api::Error> {
    let (project, tasks, notes) = (
        format!("/api/projects/{id}"),
        format!("/api/tasks?project_id={id}"),
        format!("/api/projects/{id}/notes"),
    );
    let (project, tasks, notes) = try_join!(
        api::get::<Project>(&project),
        api::all::<Task>(&tasks),
        api::get::<Vec<ProjectNote>>(&notes),
    )?;
    let area = api::get::<Area>(&format!("/api/areas/{}", project.area_id))
        .await
        .ok();
    Ok(ProjectData {
        project,
        area,
        tasks,
        notes,
    })
}

#[component]
pub fn ProjectScreen(id: i64) -> impl IntoView {
    let c = ctx();
    let data = LocalResource::new(move || {
        c.version.track();
        load_project(id)
    });
    view! {
        <Nav back="Projects" href=Route::Para(ParaTab::Projects).href()>
            <IconLink icon=icons::ARCHIVE label="Archive" href=format!("/#/projects/{id}") />
            <IconLink icon=icons::ELLIPSIS label="More" href=format!("/#/projects/{id}") />
        </Nav>
        {move || match data.get() {
            None => view! { <Loading /> }.into_any(),
            Some(Err(e)) => view! { <Failed error=e /> }.into_any(),
            Some(Ok(d)) => project(d).into_any(),
        }}
    }
}

fn project(d: ProjectData) -> impl IntoView {
    let c = ctx();
    let p = d.project.clone();
    let status = p.status().to_string();
    let total = d.tasks.len();
    let finished = d.tasks.iter().filter(|t| t.done).count();
    let percent = (finished * 100).checked_div(total).unwrap_or(0);
    let mut tasks = d.tasks.clone();
    // done first, as ticked off; then the rest by date
    tasks.sort_by(|a, b| {
        b.done.cmp(&a.done).then_with(|| {
            a.due_on
                .as_deref()
                .unwrap_or("9999")
                .cmp(b.due_on.as_deref().unwrap_or("9999"))
        })
    });
    let task_rows = tasks.iter().map(|t| task_row(t, None)).collect();
    let note_rows = d
        .notes
        .iter()
        .map(|n| {
            view! {
                <a class="row note-row press" href=Route::Note(n.zid.clone()).href()>
                    <span class="zid mono">{n.zid.clone()}</span>
                    <div class="main grow">
                        <span class="title">{n.title.clone()}</span>
                        <div class="meta">
                            <Pill kind=n.kind.clone() />
                            {if n.tags.is_empty() {
                                n.context.clone().map(|c| view! { <span>{c}</span> }).into_any()
                            } else {
                                let tags = n.tags.iter().map(|t| format!("#{t}")).collect::<Vec<_>>().join(" ");
                                view! { <span class="tags">{tags}</span> }.into_any()
                            }}
                        </div>
                    </div>
                    <Chevron />
                </a>
            }
            .into_any()
        })
        .collect();
    let due = p.due_on.as_deref().and_then(|due| {
        let day = dates::Day::parse(due)?;
        let left = dates::days_until(due, c.today).unwrap_or(0);
        let when = match left {
            0 => "today".to_string(),
            1 => "1 day".to_string(),
            n if n > 0 => format!("{n} days"),
            n => format!("{} days late", -n),
        };
        Some(format!("Due {} · {when}", day.full()))
    });
    view! {
        <div class="scroll detail">
            <div class="project-head">
                <div class="crumb">
                    <span class="area">{d.area.map(|a| a.name).unwrap_or_default()}</span>
                    <span class=format!("status-{status}")>{format!("· {status}")}</span>
                </div>
                <h1>{p.name.clone()}</h1>
                {due.map(|d| view! { <div class="due"><Ic icon=icons::CALENDAR size=16 />{d}</div> })}
            </div>
            <div class="progress">
                <div class="top"><b>"Tasks"</b><span class="muted">{format!("{finished} of {total} done")}</span></div>
                <div class="bar"><div style=format!("width:{percent}%")></div></div>
            </div>
            <Section title="Tasks">
                <a class="action" href=format!("/#/projects/{}", p.id)>"Add"</a>
            </Section>
            <Card rows=task_rows empty="No tasks yet." />
            <Section title="Notes on this project" />
            <Card rows=note_rows empty="No notes pulled in yet." />
        </div>
    }
}
