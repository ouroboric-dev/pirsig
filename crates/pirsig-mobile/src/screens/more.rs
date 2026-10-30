//! 08 More: the sections the phone hands to the full page (`desktop/dist`),
//! which works at phone width too.

use futures::join;
use leptos::prelude::*;

use crate::{
    api,
    icons::{self, Icon},
    model::{Feed, Hippo, Readiness},
    ui::{Card, Chevron, Header, Ic, Section},
};

fn row(icon: Icon, label: &'static str, detail: String, href: &'static str) -> AnyView {
    view! {
        <a class="row dense press" href=href>
            <span class="tile"><Ic icon=icon size=18 /></span>
            <span class="more-label">{label}</span>
            <span class="more-detail">{detail}</span>
            <Chevron />
        </a>
    }
    .into_any()
}

#[component]
pub fn More() -> impl IntoView {
    let counts = LocalResource::new(|| async {
        let (hippos, feeds, ready) = join!(
            api::get::<Vec<Hippo>>("/api/hippos"),
            api::get::<Vec<Feed>>("/api/feeds"),
            api::get::<Readiness>("/api/readiness"),
        );
        (
            hippos.map(|h| h.len().to_string()).unwrap_or_default(),
            feeds
                .map(|f| {
                    if f.len() == 1 {
                        "1 feed".into()
                    } else {
                        format!("{} feeds", f.len())
                    }
                })
                .unwrap_or_default(),
            ready
                .map(|r| format!("{}/{}", r.ready, r.total))
                .unwrap_or_default(),
        )
    });
    let count = move |i: usize| {
        counts
            .get()
            .map(|(h, f, r)| [h, f, r][i].clone())
            .unwrap_or_default()
    };
    view! {
        <Header title="More" />
        <div class="scroll">
            <Section title="Brain" />
            {move || view! { <Card rows=vec![row(icons::SPARKLES, "Hippos", count(0), "/#/hippos")] /> }}
            <Section title="In and out" />
            {move || view! { <Card rows=vec![row(icons::RSS, "Feeds and arXiv", count(1), "/#/feeds")] /> }}
            <Section title="Practice" />
            {move || view! {
                <Card rows=vec![
                    row(icons::DUMBBELL, "Plasticity", String::new(), "/#/plasticity"),
                    row(icons::GAUGE, "Readiness", count(2), "/#/ready"),
                ] />
            }}
            <div class="colophon">
                <span class="mu" lang="ja">"無"</span>
                <span>"Pirsig · a second brain"</span>
            </div>
        </div>
    }
}
