//! Pirsig as an app, on macOS and Android. The app runs Pirsig's own REST API
//! (the Loco app, on a free port of 127.0.0.1) and opens a window on it; the
//! API also serves the page (`desktop/dist`), so the window and the API share
//! an origin. On Android the window opens `/mobile/`, the phone UI built from
//! `crates/pirsig-mobile`. See `desktop/README.md`.

use std::{
    error::Error,
    net::TcpListener,
    path::{Path, PathBuf},
    time::Duration,
};

use include_dir::{include_dir, Dir};
use loco_rs::{
    app::Hooks,
    boot::{self, ServeParams, StartMode},
    environment::{resolve_from_env, Environment},
};
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};
use zettelkasten::{
    app::App,
    reset, seed,
    settings::{self, Seed, Settings},
};

/// What an app with no checkout runs from, unpacked into its data folder.
static DIST: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../dist");
const CONFIG: &str = include_str!("../../../config/development.yaml");
const PIRSIG_TOML: &str = include_str!("../../../pirsig.toml");

/// A checkout to run in: `PIRSIG_HOME`, else (on the desktop) the current
/// folder when it holds Pirsig's `config/`, else, in a debug build, the
/// checkout this app was built from. None: an installed app, or Android.
fn checkout() -> Option<PathBuf> {
    if let Some(home) = std::env::var_os("PIRSIG_HOME") {
        return Some(home.into());
    }
    if cfg!(mobile) {
        return None;
    }
    let cwd = std::env::current_dir().unwrap_or_default();
    if cwd.join("config").is_dir() {
        return Some(cwd);
    }
    let built_from = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    (cfg!(debug_assertions) && built_from.join("config").is_dir()).then_some(built_from)
}

/// Make `home` an app's own Pirsig folder: the config and the page as built
/// into this app (replaced on every launch, so an update brings its own), and
/// a `pirsig.toml` written once and the person's to edit after.
fn unpack(home: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(home.join("config"))?;
    std::fs::write(home.join("config/development.yaml"), CONFIG)?;
    let dist = home.join("desktop/dist");
    if dist.is_dir() {
        std::fs::remove_dir_all(&dist)?;
    }
    std::fs::create_dir_all(&dist)?;
    DIST.extract(&dist)?;
    let toml = home.join(settings::FILE);
    if !toml.exists() {
        std::fs::write(toml, PIRSIG_TOML)?;
    }
    Ok(())
}

/// A port nobody is listening on.
fn free_port() -> std::io::Result<u16> {
    Ok(TcpListener::bind("127.0.0.1:0")?.local_addr()?.port())
}

/// What this platform's empty brain starts from.
fn seed_of(settings: &Settings) -> Seed {
    if cfg!(target_os = "android") {
        settings.apps.android.seed
    } else {
        settings.apps.macos.seed
    }
}

/// Boot the API on `port` and serve it in the background; returns once it
/// answers, with the task that ends when the API stops. An app with no
/// checkout fills an empty brain from `[template] repo` first, when its
/// `seed` says so; a failed download leaves the brain empty, not the app shut.
async fn serve(
    port: u16,
    settings: &Settings,
    may_seed: bool,
) -> Result<tokio::task::JoinHandle<()>, Box<dyn Error>> {
    let environment: Environment = resolve_from_env().into();
    let mut config = App::load_config(&environment).await?;
    config.server.port = port.into();
    config.server.binding = "127.0.0.1".into();
    config.server.host = "http://127.0.0.1".into();
    let booted = App::boot(StartMode::ServerOnly, &environment, config).await?;
    let db = &booted.app_context.db;
    if may_seed && seed_of(settings) == Seed::Template && reset::is_empty(db).await? {
        match seed::bootstrap(db, settings, None, None).await {
            Ok(r) => eprintln!(
                "Pirsig: bootstrapped from {}: {} notes, {} resources",
                settings.template.repo, r.notes_created, r.resources
            ),
            Err(e) => eprintln!("Pirsig: no bootstrap from {}: {e}", settings.template.repo),
        }
    }
    let params = ServeParams {
        port: port.into(),
        binding: "127.0.0.1".into(),
    };
    let server = tokio::spawn(async move {
        if let Err(e) = boot::start::<App>(booted, params, true).await {
            eprintln!("Pirsig API stopped: {e}");
        }
    });
    for _ in 0..200 {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            return Ok(server);
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    Err("the Pirsig API did not start".into())
}

/// # Panics
/// When the API cannot start, or Tauri cannot open the window.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // the API lives on this runtime, which the app keeps for as long as it
    // runs (on Android, `run` returns while the app goes on)
    let runtime = tokio::runtime::Runtime::new().expect("a tokio runtime");
    tauri::Builder::default()
        .setup(move |app| {
            let checkout = checkout();
            let home = if let Some(dir) = &checkout {
                dir.clone()
            } else {
                let dir = app.path().app_data_dir()?;
                unpack(&dir)?;
                dir
            };
            std::env::set_current_dir(&home)?;
            let settings = Settings::read(&home.join(settings::FILE))?;
            let port = free_port()?;
            let server = runtime.block_on(serve(port, &settings, checkout.is_none()))?;
            // the API stops on a shutdown signal (Ctrl-C, SIGTERM) or an
            // error; a window with nothing behind it is no use, so quit too
            let app_handle = app.handle().clone();
            runtime.spawn(async move {
                let _ = server.await;
                app_handle.exit(0);
            });
            app.manage(runtime);
            // a phone gets its own page (crates/pirsig-mobile, built into dist/mobile)
            let page = if cfg!(mobile) { "mobile/" } else { "" };
            let url = format!("http://127.0.0.1:{port}/{page}").parse()?;
            let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::External(url));
            #[cfg(desktop)]
            let window = window
                .title("Pirsig")
                .inner_size(settings.apps.width.into(), settings.apps.height.into())
                .min_inner_size(720.0, 480.0);
            window.build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("Pirsig could not start");
}
