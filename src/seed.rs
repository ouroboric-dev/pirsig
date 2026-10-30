//! Bootstrapping a brain from a Pirsig repository: the fork state (`brain/`)
//! of the template, of any copy of it, or of a local checkout.
//!
//! `psg bootstrap repo` and the apps' first launch use this. A GitHub
//! repository is read from its tarball (one HTTPS request, no `git`), and only
//! the fork state's folder is unpacked, into a scratch folder that is removed
//! after the import.
#![allow(clippy::missing_errors_doc)]
use std::{
    io::Read,
    path::{Component, Path, PathBuf},
};

use loco_rs::{Error, Result};
use sea_orm::DatabaseConnection;

use crate::{
    ingest,
    settings::Settings,
    vault::{self, ImportReport},
};

/// Where a fork state comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// A checkout on this machine.
    Dir(PathBuf),
    /// `owner/repo` on GitHub.
    Github { owner: String, repo: String },
}

impl Source {
    /// `owner/repo`, `https://github.com/owner/repo(.git)`, or a folder: a
    /// path that exists, or that starts with `/`, `.` or `~`.
    pub fn parse(repo: &str) -> Result<Self> {
        let repo = repo.trim();
        let path = Path::new(repo);
        if let Some(rest) = repo.strip_prefix("~/") {
            let home = std::env::var_os("HOME").unwrap_or_default();
            return Ok(Self::Dir(Path::new(&home).join(rest)));
        }
        if path.is_dir() || repo.starts_with(['/', '.']) {
            return Ok(Self::Dir(path.into()));
        }
        let slug = repo
            .strip_prefix("https://github.com/")
            .or_else(|| repo.strip_prefix("github.com/"))
            .unwrap_or(repo)
            .trim_end_matches('/');
        let slug = slug.strip_suffix(".git").unwrap_or(slug);
        let ok = |s: &str| {
            !s.is_empty()
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        };
        match slug.split_once('/') {
            Some((owner, name)) if ok(owner) && ok(name) => Ok(Self::Github {
                owner: owner.into(),
                repo: name.into(),
            }),
            _ => Err(Error::Message(format!(
                "{repo:?} is not owner/repo, a GitHub URL or a folder"
            ))),
        }
    }
}

/// Import the fork state of `[template] repo` (or `repo`, when given) into
/// `db`. The caller decides what to do with a brain that is not empty.
pub async fn bootstrap(
    db: &DatabaseConnection,
    settings: &Settings,
    repo: Option<&str>,
    branch: Option<&str>,
) -> Result<ImportReport> {
    let source = Source::parse(repo.unwrap_or(&settings.template.repo))?;
    let fork_state = &settings.brain.fork_state;
    match source {
        Source::Dir(dir) => {
            let dir = dir.join(fork_state);
            if !dir.is_dir() {
                return Err(Error::Message(format!(
                    "{} is not a folder: is this a Pirsig checkout?",
                    dir.display()
                )));
            }
            vault::import(db, &dir, false).await
        }
        Source::Github { owner, repo } => {
            let branch = branch.unwrap_or(&settings.template.branch);
            let url =
                format!("https://codeload.github.com/{owner}/{repo}/tar.gz/refs/heads/{branch}");
            let tarball = ingest::fetch(&url)
                .await
                .map_err(|e| Error::Message(format!("{owner}/{repo}@{branch}: {e}")))?;
            let scratch = std::env::temp_dir().join(format!(
                "pirsig-seed-{}-{}",
                std::process::id(),
                chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
            ));
            let unpacked = unpack(tarball.as_slice(), fork_state, &scratch);
            let report = match unpacked {
                Ok(0) => Err(Error::Message(format!(
                    "{owner}/{repo}@{branch} has no {}/: is it a Pirsig repository?",
                    fork_state.display()
                ))),
                Ok(_) => vault::import(db, &scratch, false).await,
                Err(e) => Err(e),
            };
            let _ = std::fs::remove_dir_all(&scratch);
            report
        }
    }
}

/// Unpack the files under `dir` of a GitHub tarball (whose entries all sit in
/// one `<repo>-<branch>/` folder) into `to`; returns how many. Entries that
/// would land outside `to` are skipped.
pub fn unpack(tar_gz: impl Read, dir: &Path, to: &Path) -> Result<usize> {
    let io = |e: std::io::Error| Error::Message(format!("unpacking the repository: {e}"));
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(tar_gz));
    let mut count = 0;
    for entry in archive.entries().map_err(io)? {
        let mut entry = entry.map_err(io)?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = entry.path().map_err(io)?.into_owned();
        let mut parts = path.components();
        parts.next(); // <repo>-<branch>/
        let Ok(rest) = parts.as_path().strip_prefix(dir) else {
            continue;
        };
        if rest.as_os_str().is_empty()
            || !rest.components().all(|c| matches!(c, Component::Normal(_)))
        {
            continue;
        }
        let out = to.join(rest);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(io)?;
        }
        entry.unpack(&out).map_err(io)?;
        count += 1;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sources() {
        let gh = |o: &str, r: &str| Source::Github {
            owner: o.into(),
            repo: r.into(),
        };
        assert_eq!(
            Source::parse("h0ffmann/pirsig").unwrap(),
            gh("h0ffmann", "pirsig")
        );
        assert_eq!(
            Source::parse("https://github.com/acme/brain.git").unwrap(),
            gh("acme", "brain")
        );
        assert_eq!(
            Source::parse("./my-copy").unwrap(),
            Source::Dir("./my-copy".into())
        );
        assert!(Source::parse("not a repo").is_err());
        assert!(Source::parse("a/b/c").is_err());
    }

    #[test]
    fn unpacks_only_the_fork_state() {
        let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::fast(),
        ));
        for (path, body) in [
            ("pirsig-main/README.md", "readme"),
            ("pirsig-main/brain/Zettelkasten/1 A note.md", "a note"),
            ("pirsig-main/brain/Areas/Health.md", "health"),
            ("pirsig-main/brainstorm/x.md", "not the fork state"),
        ] {
            let mut header = tar::Header::new_gnu();
            header.set_size(body.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            tar.append_data(&mut header, path, body.as_bytes()).unwrap();
        }
        let tar_gz = tar.into_inner().unwrap().finish().unwrap();
        let to = std::env::temp_dir().join(format!("pirsig-seed-test-{}", std::process::id()));
        let n = unpack(tar_gz.as_slice(), Path::new("brain"), &to).unwrap();
        assert_eq!(n, 2);
        assert_eq!(
            std::fs::read_to_string(to.join("Zettelkasten/1 A note.md")).unwrap(),
            "a note"
        );
        assert!(to.join("Areas/Health.md").is_file());
        assert!(!to.join("README.md").exists());
        std::fs::remove_dir_all(&to).unwrap();
    }
}
