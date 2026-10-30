//! `pirsig.toml`: how a copy of Pirsig runs — where a new brain comes from, the
//! defaults of `psg gh`, and what the apps do on first launch.
//!
//! Every key is optional and takes the default the repository's own
//! `pirsig.toml` shows; a key Pirsig does not know is an error, so a typo is
//! not silently ignored. The Loco side (port, database) stays in `config/`.
#![allow(clippy::missing_errors_doc)]
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use loco_rs::{Error, Result};
use serde::{Deserialize, Serialize};

/// The file `psg` reads, in the folder it runs in.
pub const FILE: &str = "pirsig.toml";
/// Names another file to read instead.
pub const ENV: &str = "PIRSIG_CONFIG";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    pub template: Template,
    pub brain: Brain,
    pub github: Github,
    pub apps: Apps,
}

/// The repository a new brain is bootstrapped from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Template {
    /// `owner/repo`, a GitHub URL, or a local folder (a checkout).
    pub repo: String,
    pub branch: String,
}

impl Default for Template {
    fn default() -> Self {
        Self {
            repo: "h0ffmann/pirsig".into(),
            branch: "main".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Brain {
    /// The fork state's folder in the repository.
    pub fork_state: PathBuf,
}

impl Default for Brain {
    fn default() -> Self {
        Self {
            fork_state: "brain".into(),
        }
    }
}

/// Defaults of `psg gh` and `psg bootstrap gh`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Github {
    /// Empty: the token's user.
    pub user: String,
    pub private: PrivateRepos,
    pub forks: Forks,
    pub cold_after: u32,
    pub readme: bool,
}

impl Default for Github {
    fn default() -> Self {
        Self {
            user: String::new(),
            private: PrivateRepos::Skip,
            forks: Forks::Index,
            cold_after: 24,
            readme: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Forks {
    /// One "GitHub: public forks" note, a line per fork.
    Index,
    /// A note per fork, as for own repositories.
    Notes,
    /// Leave forks out.
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum PrivateRepos {
    /// Leave them out.
    Skip,
    /// Name, URL, language and last push; never description, topics or README.
    Minimal,
}

/// The Tauri apps: macOS desktop and Android.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Apps {
    pub width: u32,
    pub height: u32,
    pub macos: App,
    pub android: App,
}

impl Default for Apps {
    fn default() -> Self {
        Self {
            width: 1240,
            height: 820,
            macos: App {
                seed: Seed::Template,
            },
            android: App { seed: Seed::Empty },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct App {
    pub seed: Seed,
}

/// What an empty brain starts from when an app first opens it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Seed {
    /// The fork state of `[template] repo`.
    Template,
    /// Nothing.
    Empty,
}

impl Settings {
    /// Read `text` as a `pirsig.toml`.
    pub fn parse(text: &str) -> Result<Self> {
        toml::from_str(text).map_err(|e| Error::Message(format!("{FILE}: {e}")))
    }

    /// Read `path`; a missing file means every default.
    pub fn read(path: &Path) -> Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(text) => {
                Self::parse(&text).map_err(|e| Error::Message(format!("{}: {e}", path.display())))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(Error::Message(format!("{}: {e}", path.display()))),
        }
    }

    /// The file `PIRSIG_CONFIG` names, else `./pirsig.toml`.
    pub fn load() -> Result<Self> {
        let path = std::env::var_os(ENV).map_or_else(|| PathBuf::from(FILE), PathBuf::from);
        Self::read(&path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_repositorys_file_is_the_defaults() {
        let shipped = include_str!("../pirsig.toml");
        assert_eq!(Settings::parse(shipped).unwrap(), Settings::default());
    }

    #[test]
    fn every_key_is_optional() {
        assert_eq!(Settings::parse("").unwrap(), Settings::default());
        let s = Settings::parse("[github]\ncold_after = 6\n[apps.android]\nseed = \"template\"")
            .unwrap();
        assert_eq!(s.github.cold_after, 6);
        assert_eq!(s.github.forks, Forks::Index);
        assert_eq!(s.apps.android.seed, Seed::Template);
        assert_eq!(s.apps.macos.seed, Seed::Template);
    }

    #[test]
    fn a_typo_is_an_error() {
        let err = Settings::parse("[github]\ncold_afer = 6").unwrap_err();
        assert!(err.to_string().contains("cold_afer"), "{err}");
        assert!(Settings::parse("[apps.macos]\nseed = \"maybe\"").is_err());
    }

    #[test]
    fn a_missing_file_is_the_defaults() {
        let s = Settings::read(Path::new("no/such/pirsig.toml")).unwrap();
        assert_eq!(s, Settings::default());
    }
}
