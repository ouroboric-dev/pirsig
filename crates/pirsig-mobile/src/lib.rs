//! Pirsig on a phone: the Android app's page, in Rust. The Tauri shell
//! (`desktop/src-tauri`) runs Pirsig's API and opens its window on
//! `/mobile/`, which `scripts/build-mobile-ui.sh` fills with this crate as
//! WebAssembly. The screens follow the mobile design drawn in `OpenPencil`
//! (`docs/design/pirsig-mobile.fig`): its variables are `style/tokens.css`, its
//! icons `src/icons.rs`, and each component is named after its frame.

// A `#[component]` is a view, built by `view!`; its props are owned by design.
#![allow(clippy::must_use_candidate, clippy::needless_pass_by_value)]

pub mod api;
pub mod dates;
pub mod icons;
pub mod md;
pub mod model;
pub mod route;
pub mod screens;
pub mod ui;

use leptos::prelude::*;
use wasm_bindgen::{prelude::*, JsCast};

use crate::{
    dates::Day,
    route::Route,
    screens::{
        capture::CaptureSheet,
        more::More,
        notes::{NoteScreen, Notes},
        para::{Para, ProjectScreen},
        slipbox::SlipBox,
        today::Today,
    },
    ui::{Ctx, TabBar},
};

fn hash() -> String {
    window().location().hash().unwrap_or_default()
}

/// Follow the system theme: `data-pirsig="dark"` switches the tokens.
fn follow_theme() {
    let Ok(Some(query)) = window().match_media("(prefers-color-scheme: dark)") else {
        return;
    };
    let apply = |dark: bool| {
        if let Some(root) = document().document_element() {
            let _ = if dark {
                root.set_attribute("data-pirsig", "dark")
            } else {
                root.remove_attribute("data-pirsig")
            };
        }
    };
    apply(query.matches());
    let on_change = Closure::<dyn Fn(web_sys::MediaQueryListEvent)>::new(
        move |ev: web_sys::MediaQueryListEvent| apply(ev.matches()),
    );
    let _ = query.add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref());
    on_change.forget();
}

#[component]
fn App() -> impl IntoView {
    let route = RwSignal::new(Route::parse(&hash()));
    let listener = window_event_listener(leptos::ev::hashchange, move |_| {
        route.set(Route::parse(&hash()));
    });
    on_cleanup(move || listener.remove());

    let now = js_sys::Date::new_0();
    #[allow(clippy::cast_possible_truncation)] // the epoch in seconds fits an i64 for a while yet
    let secs = (now.get_time() / 1000.0) as i64;
    let ctx = Ctx {
        capture: RwSignal::new(false),
        version: RwSignal::new(0),
        toast: RwSignal::new(None),
        today: Day {
            year: i32::try_from(now.get_full_year()).unwrap_or(1970),
            month: now.get_month() + 1,
            day: now.get_date(),
        },
        now: secs,
    };
    provide_context(ctx);

    // a new screen starts at its top
    let screen = NodeRef::<leptos::html::Main>::new();
    Effect::new(move |_| {
        route.track();
        if let Some(el) = screen.get_untracked() {
            el.set_scroll_top(0);
        }
    });

    view! {
        <main class="screen" node_ref=screen>
            {move || match route.get() {
                Route::Today | Route::NotFound => view! { <Today /> }.into_any(),
                Route::Notes(kind) => view! { <Notes kind=kind /> }.into_any(),
                Route::Note(zid) => view! { <NoteScreen zid=zid /> }.into_any(),
                Route::SlipBox(tab) => view! { <SlipBox tab=tab /> }.into_any(),
                Route::Para(tab) => view! { <Para tab=tab /> }.into_any(),
                Route::Project(id) => view! { <ProjectScreen id=id /> }.into_any(),
                Route::More => view! { <More /> }.into_any(),
            }}
        </main>
        // a note reads full height, as the design's Note frame has no tab bar
        <Show when=move || !matches!(route.get(), Route::Note(_))>
            <TabBar on=Signal::derive(move || route.get().tab()) />
        </Show>
        <Show when=move || ctx.capture.get()>
            <CaptureSheet />
        </Show>
        {move || ctx.toast.get().map(|(text, bad)| view! { <div class="toast" class:bad=bad role="status">{text}</div> })}
    }
}

/// The page's entry point, run when the WebAssembly module loads.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    follow_theme();
    let root = document()
        .get_element_by_id("app")
        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok());
    if let Some(root) = root {
        leptos::mount::mount_to(root, App).forget();
    }
}
