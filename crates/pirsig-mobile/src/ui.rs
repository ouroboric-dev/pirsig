//! The pieces every screen is made of, each named after its frame in the
//! mobile design (OpenPencil): Header, Nav, Section, Card, Row, Pill, …

use leptos::prelude::*;

use crate::{
    dates::{self, Day},
    icons::{self, Icon},
    md::{self, Block, Inline},
    model::{Link, Note},
    route::{Route, Tab},
};

/// What every screen shares: the time, and a way to ask the others to reload.
#[derive(Clone, Copy)]
pub struct Ctx {
    pub capture: RwSignal<bool>,
    /// Bumped after a change, so resources that read it fetch again.
    pub version: RwSignal<u32>,
    pub toast: RwSignal<Option<(String, bool)>>,
    pub today: Day,
    pub now: i64,
}

impl Ctx {
    pub fn changed(&self) {
        self.version.update(|v| *v += 1);
    }

    pub fn say(&self, text: impl Into<String>, bad: bool) {
        let toast = self.toast;
        toast.set(Some((text.into(), bad)));
        set_timeout(
            move || toast.set(None),
            std::time::Duration::from_millis(2800),
        );
    }
}

#[must_use]
pub fn ctx() -> Ctx {
    expect_context()
}

/// An icon at `size` px, in the text colour around it.
#[component]
pub fn Ic(
    icon: Icon,
    #[prop(default = 22)] size: u32,
    #[prop(optional, into)] class: String,
) -> impl IntoView {
    let svg = format!(
        r#"<svg viewBox="0 0 24 24" aria-hidden="true">{}</svg>"#,
        icon.0
    );
    view! { <span class=format!("ic {class}") style=format!("width:{size}px;height:{size}px") inner_html=svg></span> }
}

/// A screen's title, its actions on the right, and a line under it.
#[component]
pub fn Header(
    #[prop(into)] title: String,
    #[prop(optional, into)] sub: Option<Signal<String>>,
    #[prop(optional)] accent: bool,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <header class="header">
            <div class="top">
                <h1>{title}</h1>
                <div class="actions" class:accent=accent>{children.map(|c| c())}</div>
            </div>
            {sub.map(|s| view! { <p class="sub">{move || s.get()}</p> })}
        </header>
    }
}

/// Back to a section, with actions on the right.
#[component]
pub fn Nav(
    #[prop(into)] back: String,
    href: String,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <nav class="nav">
            <a class="back press" href=href>
                <Ic icon=icons::CHEVRON_LEFT size=26 />
                {back}
            </a>
            <div class="actions">{children.map(|c| c())}</div>
        </nav>
    }
}

/// An icon that does something, with a 44px target around it.
#[component]
pub fn IconButton(
    icon: Icon,
    #[prop(into)] label: String,
    #[prop(default = 22)] size: u32,
    on: Callback<()>,
) -> impl IntoView {
    view! {
        <button class="icon-button press" aria-label=label on:click=move |_| on.run(())>
            <Ic icon=icon size=size />
        </button>
    }
}

/// An icon that goes somewhere, with a 44px target around it.
#[component]
pub fn IconLink(
    icon: Icon,
    #[prop(into)] label: String,
    #[prop(default = 22)] size: u32,
    href: String,
) -> impl IntoView {
    view! {
        <a class="icon-button press" aria-label=label href=href>
            <Ic icon=icon size=size />
        </a>
    }
}

#[component]
pub fn Section(
    #[prop(into)] title: String,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="section">
            <h2>{title}</h2>
            {children.map(|c| c())}
        </div>
    }
}

/// A card of rows; `empty` shows when it has none.
#[component]
pub fn Card(#[prop(optional, into)] empty: Option<String>, rows: Vec<AnyView>) -> impl IntoView {
    let body = if rows.is_empty() {
        view! { <p class="empty">{empty.unwrap_or_default()}</p> }.into_any()
    } else {
        rows.into_iter().collect_view().into_any()
    };
    view! { <div class="card">{body}</div> }
}

#[component]
pub fn Pill(#[prop(into)] kind: String) -> impl IntoView {
    view! { <span class=format!("pill {kind}")>{kind.clone()}</span> }
}

#[component]
pub fn Chevron() -> impl IntoView {
    view! { <Ic icon=icons::CHEVRON_RIGHT size=18 class="chev" /> }
}

/// A note in a list: zid, title, kind, and its tags or when it changed.
pub fn note_row(n: &Note, trail: Option<String>) -> AnyView {
    row_of(n, trail, false)
}

/// A note waiting in the inbox: when it changed, whatever its tags.
pub fn inbox_row(n: &Note) -> AnyView {
    row_of(n, None, true)
}

fn row_of(n: &Note, trail: Option<String>, aged: bool) -> AnyView {
    let c = ctx();
    let tagged = !aged && !n.tags.is_empty();
    let meta = if tagged {
        n.tags
            .iter()
            .map(|t| format!("#{t}"))
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        dates::ago(&n.updated_at, c.now, c.today)
    };
    view! {
        <a class="row note-row press" href=Route::Note(n.zid.clone()).href()>
            <span class="zid mono">{n.zid.clone()}</span>
            <div class="main grow">
                <span class="title">{n.title.clone()}</span>
                <div class="meta">
                    <Pill kind=n.kind.clone() />
                    <span class:tags=tagged>{meta}</span>
                </div>
            </div>
            {match trail {
                Some(t) => view! { <span class="trail">{t}</span> }.into_any(),
                None => view! { <Chevron /> }.into_any(),
            }}
        </a>
    }
    .into_any()
}

/// A tappable row that goes to another note, as on the Note screen.
pub fn link_row(zid: &str, title: &str) -> AnyView {
    view! {
        <a class="row link press" href=Route::Note(zid.to_string()).href()>
            <span class="zid mono">{zid.to_string()}</span>
            <span class="link-title grow">{title.to_string()}</span>
            <Chevron />
        </a>
    }
    .into_any()
}

/// Two to four choices side by side, the current one raised.
pub fn segments<T: Copy + PartialEq + 'static>(items: &[(T, String, String)], on: T) -> AnyView {
    let views = items
        .iter()
        .map(|(value, label, href)| {
            let current = *value == on;
            view! {
                <a class="segment" class:on=current href=href.clone() aria-current=current.then_some("page")>
                    {label.clone()}
                </a>
            }
        })
        .collect_view();
    view! { <nav class="segments">{views}</nav> }.into_any()
}

/// The tab bar: Today, Notes, Slip-box, PARA, More.
#[component]
pub fn TabBar(on: Signal<Tab>) -> impl IntoView {
    let tabs = [
        (Tab::Today, icons::SUN, "Today", Route::Today),
        (Tab::Notes, icons::NOTEBOOK_PEN, "Notes", Route::Notes(None)),
        (
            Tab::SlipBox,
            icons::INBOX,
            "Slip-box",
            Route::SlipBox(crate::route::SlipTab::Inbox),
        ),
        (
            Tab::Para,
            icons::FOLDER_KANBAN,
            "PARA",
            Route::Para(crate::route::ParaTab::Projects),
        ),
        (Tab::More, icons::MENU, "More", Route::More),
    ];
    view! {
        <nav class="tabbar" aria-label="Sections">
            {tabs
                .into_iter()
                .map(|(tab, icon, label, route)| {
                    view! {
                        <a href=route.href() class:on=move || on.get() == tab aria-current=move || (on.get() == tab).then_some("page")>
                            <Ic icon=icon size=22 />
                            {label}
                        </a>
                    }
                })
                .collect_view()}
        </nav>
    }
}

/// Grey blocks where the cards will be, while the API answers.
#[component]
pub fn Loading() -> impl IntoView {
    view! {
        <div class="scroll" aria-busy="true">
            <div class="skeleton"></div>
            <div class="skeleton"></div>
        </div>
    }
}

#[component]
pub fn Failed(error: String) -> impl IntoView {
    view! { <div class="scroll"><p class="error">{error}</p></div> }
}

/// A note's body, from [`md::parse`], with wikilinks resolved among its links.
pub fn body(text: &str, links: &[Link]) -> AnyView {
    let blocks = md::parse(text)
        .into_iter()
        .map(|b| match b {
            Block::Heading(i) => view! { <h3>{inlines(i, links)}</h3> }.into_any(),
            // a line of only #tags is the note's tag line
            Block::Paragraph(i) if only_tags(&i) => view! { <p class="tagline">{inlines(i, links)}</p> }.into_any(),
            Block::Paragraph(i) => view! { <p>{inlines(i, links)}</p> }.into_any(),
            Block::Quote(i) => view! { <blockquote><span>{inlines(i, links)}</span></blockquote> }.into_any(),
            Block::List(items) => view! {
                <ul>{items.into_iter().map(|i| view! { <li>{inlines(i, links)}</li> }).collect_view()}</ul>
            }
            .into_any(),
            Block::Code(c) => view! { <pre class="mono">{c}</pre> }.into_any(),
            Block::Rule => view! { <hr /> }.into_any(),
        })
        .collect_view();
    view! { <div class="body">{blocks}</div> }.into_any()
}

fn only_tags(items: &[Inline]) -> bool {
    items.iter().any(|i| matches!(i, Inline::Tag(_)))
        && items.iter().all(|i| match i {
            Inline::Tag(_) => true,
            Inline::Text(t) => t.trim().is_empty(),
            _ => false,
        })
}

fn inlines(items: Vec<Inline>, links: &[Link]) -> AnyView {
    items
        .into_iter()
        .map(|i| match i {
            Inline::Text(t) => t.into_any(),
            Inline::Code(c) => view! { <code class="mono">{c}</code> }.into_any(),
            Inline::Strong(s) => view! { <strong>{s}</strong> }.into_any(),
            Inline::Em(s) => view! { <em>{s}</em> }.into_any(),
            Inline::Tag(t) => view! { <span class="tag">{format!("#{t}")}</span> }.into_any(),
            Inline::Link { href, text } => view! { <a class="wikilink" href=href target="_blank" rel="noopener">{text}</a> }.into_any(),
            Inline::Wiki { target, text } => {
                let key = target.to_lowercase();
                match links.iter().find(|l| l.zid == target || l.title.to_lowercase() == key) {
                    Some(l) => {
                        // `[[1a]]` alone reads as the note's title, its zid after it
                        let shown = if text == l.zid { l.title.clone() } else { text };
                        view! {
                            <a class="wikilink" href=Route::Note(l.zid.clone()).href()>{shown}</a>
                            <span class="wikilink"><span class="zid-tail mono">{format!("[[{}]]", l.zid)}</span></span>
                        }
                        .into_any()
                    }
                    None => view! { <span class="wikilink missing">{text}</span> }.into_any(),
                }
            }
        })
        .collect_view()
        .into_any()
}
