use std::path::PathBuf;

use clap::Subcommand;
use loco_rs::{app::AppContext, Error, Result};

use super::{obsidian::exported, Output};
use crate::{reset, settings::Settings, vault};

/// `psg fork export`, `psg fork import --overwrite`.
#[derive(Debug, clap::Args)]
pub struct Args {
    #[command(subcommand)]
    action: Action,
}

#[derive(Debug, Subcommand)]
enum Action {
    /// Write the brain as the fork state other copies of the repository start
    /// from: a vault in brain/, without the notes tagged #private, the
    /// resources only they cite, or any link or list line that names them.
    Export {
        /// Folder; created if missing. Only files Pirsig wrote are replaced
        /// [default: `[brain] fork_state` in pirsig.toml, brain].
        dir: Option<PathBuf>,
    },
    /// Start from the fork state: read brain/ into an empty brain.
    Import {
        /// Folder [default: `[brain] fork_state` in pirsig.toml, brain].
        dir: Option<PathBuf>,
        /// Replace a brain that is not empty: it is backed up next to its
        /// database file, emptied, then filled from the fork state.
        #[arg(long)]
        overwrite: bool,
    },
}

pub async fn run(ctx: &AppContext, out: &Output, args: Args) -> Result<()> {
    let fork_state = || Settings::load().map(|s| s.brain.fork_state);
    match args.action {
        Action::Export { dir } => {
            let dir = dir.map_or_else(fork_state, Ok)?;
            let report = vault::export_public(&ctx.db, &dir).await?;
            out.one(&report, |r| exported(r, &dir));
        }
        Action::Import { dir, overwrite } => {
            let dir = dir.map_or_else(fork_state, Ok)?;
            if !dir.is_dir() {
                return Err(Error::Message(format!(
                    "{} is not a folder: export a fork state with `psg fork export` first",
                    dir.display()
                )));
            }
            if !reset::is_empty(&ctx.db).await? {
                if !overwrite {
                    return Err(Error::Message(
                        "this brain is not empty: pass --overwrite to replace it (it is backed up first)"
                            .into(),
                    ));
                }
                let wiped = reset::wipe(&ctx.db, &ctx.config.database.uri).await?;
                if !out.json {
                    println!("{}", reset::wiped_line(&wiped));
                }
            }
            let report = vault::import(&ctx.db, &dir, false).await?;
            out.one(&report, super::obsidian::imported);
        }
    }
    Ok(())
}
