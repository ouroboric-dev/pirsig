//! 06 Capture: a sheet over any screen: a title, a body, its kind, and the
//! project and Hippo it belongs to.

use futures::try_join;
use leptos::{html, prelude::*};

use crate::{
    api, icons,
    model::{Hippo, NewNote, NoteDetail, Project, PullNote},
    screens::notes::KINDS,
    ui::{ctx, Ic},
};

/// Write the note, then pull it into the project when one was chosen.
async fn capture(note: &NewNote, project: Option<i64>) -> Result<NoteDetail, api::Error> {
    let made = api::post::<_, NoteDetail>("/api/notes", note).await?;
    if let Some(id) = project {
        let pull = PullNote {
            zid: made.note.zid.clone(),
        };
        api::post::<_, serde_json::Value>(&format!("/api/projects/{id}/notes"), &pull).await?;
    }
    Ok(made)
}

#[component]
pub fn CaptureSheet() -> impl IntoView {
    let c = ctx();
    let title = RwSignal::new(String::new());
    let body = RwSignal::new(String::new());
    let kind = RwSignal::new("fleeting".to_string());
    let project = RwSignal::new(None::<i64>);
    let hippo = RwSignal::new(None::<i64>);
    let saving = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let input = NodeRef::<html::Textarea>::new();
    Effect::new(move |_| {
        if let Some(el) = input.get() {
            let _ = el.focus();
        }
    });
    let close = move || {
        c.capture.set(false);
        title.set(String::new());
        body.set(String::new());
        kind.set("fleeting".into());
        project.set(None);
        hippo.set(None);
        error.set(None);
    };
    let save = move || {
        let note = NewNote {
            title: title.get_untracked().trim().to_string(),
            body: body.get_untracked(),
            kind: kind.get_untracked(),
            hippo_id: hippo.get_untracked(),
        };
        if note.title.is_empty() || saving.get_untracked() {
            return;
        }
        let into = project.get_untracked();
        saving.set(true);
        leptos::task::spawn_local(async move {
            let made = capture(&note, into).await;
            saving.set(false);
            match made {
                Ok(d) => {
                    let where_ = if d.note.kind == "fleeting" {
                        " in the inbox"
                    } else {
                        ""
                    };
                    c.say(
                        format!("Captured “{}”{where_} as {}", d.note.title, d.note.zid),
                        false,
                    );
                    close();
                    c.changed();
                }
                Err(e) => error.set(Some(e)),
            }
        });
    };
    view! {
        <div class="scrim" on:click=move |_| close()></div>
        <div class="sheet" role="dialog" aria-modal="true" aria-label="Capture">
            <div class="handle"></div>
            <div class="sheet-bar">
                <button on:click=move |_| close()>"Cancel"</button>
                <span class="title">"Capture"</span>
                <button class="save" disabled=move || title.with(|t| t.trim().is_empty()) || saving.get() on:click=move |_| save()>
                    {move || if saving.get() { "Saving…" } else { "Save" }}
                </button>
            </div>
            <textarea
                class="title-input"
                rows="2"
                node_ref=input
                placeholder="A thought, as a claim"
                aria-label="Title"
                prop:value=move || title.get()
                on:input:target=move |ev| title.set(ev.target().value())
                on:keydown=move |ev| {
                    // a title is one line: Enter saves
                    if ev.key() == "Enter" {
                        ev.prevent_default();
                        save();
                    }
                }
            ></textarea>
            <textarea
                placeholder="In your own words. [[links]] and #tags work."
                aria-label="Body"
                prop:value=move || body.get()
                on:input:target=move |ev| body.set(ev.target().value())
            ></textarea>
            <Kinds kind=kind />
            <Pickers project=project hippo=hippo />
            {move || error.get().map(|e| view! { <p class="error">{e}</p> })}
        </div>
    }
}

/// The four kinds as chips, the chosen one filled.
#[component]
fn Kinds(kind: RwSignal<String>) -> impl IntoView {
    let chips = KINDS
        .iter()
        .map(|&k| {
            view! {
                <button
                    class="chip bare press"
                    class:on=move || kind.with(|v| v == k)
                    aria-pressed=move || kind.with(|v| v == k).to_string()
                    on:click=move |_| kind.set(k.to_string())
                >
                    {k}
                </button>
            }
        })
        .collect_view();
    view! { <div class="chips" role="group" aria-label="Kind">{chips}</div> }
}

/// Which project the note goes into, and which Hippo it grows in.
#[component]
fn Pickers(project: RwSignal<Option<i64>>, hippo: RwSignal<Option<i64>>) -> impl IntoView {
    let pickers = LocalResource::new(move || async move {
        try_join!(
            api::all::<Project>("/api/projects"),
            api::get::<Vec<Hippo>>("/api/hippos")
        )
    });
    view! {
            <div class="pickers">
                <label class="row">
                    <Ic icon=icons::FOLDER_KANBAN size=18 class="muted" />
                    <span class="label">"Project"</span>
                    <select on:change:target=move |ev| project.set(ev.target().value().parse().ok())>
                        <option value="">"none"</option>
                        {move || pickers.get().and_then(Result::ok).map(|(projects, _)| {
                            projects
                                .into_iter()
                                .filter(|p| p.archived_at.is_none() && p.status() != "done")
                                .map(|p| view! { <option value=p.id.to_string()>{p.name}</option> })
                                .collect_view()
                        })}
                    </select>
                    <Ic icon=icons::CHEVRON_RIGHT size=16 class="muted" />
                </label>
                <label class="row">
                    <Ic icon=icons::SPARKLES size=18 class="muted" />
                    <span class="label">"Hippo"</span>
                    <select on:change:target=move |ev| hippo.set(ev.target().value().parse().ok())>
                        <option value="">"none"</option>
                        {move || pickers.get().and_then(Result::ok).map(|(_, hippos)| {
                            hippos.into_iter().map(|h| view! { <option value=h.id.to_string()>{h.name}</option> }).collect_view()
                        })}
                    </select>
                    <Ic icon=icons::CHEVRON_RIGHT size=16 class="muted" />
                </label>
            </div>
    }
}
