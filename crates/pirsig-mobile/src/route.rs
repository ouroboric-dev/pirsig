//! Where the app is: the part of the URL after `#`, as in the desktop page, so
//! a reload, the back gesture and a deep link all land on the same screen.

use std::fmt::Write as _;

/// A tab of the slip-box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlipTab {
    Inbox,
    Orphans,
    Tree,
    Tags,
}

impl SlipTab {
    pub const ALL: [Self; 4] = [Self::Inbox, Self::Orphans, Self::Tree, Self::Tags];

    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Inbox => "inbox",
            Self::Orphans => "orphans",
            Self::Tree => "tree",
            Self::Tags => "tags",
        }
    }
}

/// A tab of PARA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParaTab {
    Projects,
    Areas,
    Resources,
    Tasks,
}

impl ParaTab {
    pub const ALL: [Self; 4] = [Self::Projects, Self::Areas, Self::Resources, Self::Tasks];

    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Projects => "projects",
            Self::Areas => "areas",
            Self::Resources => "resources",
            Self::Tasks => "tasks",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Projects => "Projects",
            Self::Areas => "Areas",
            Self::Resources => "Resources",
            Self::Tasks => "Tasks",
        }
    }
}

/// A screen of the app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    Today,
    /// The notes, filtered by kind when `Some`.
    Notes(Option<String>),
    Note(String),
    SlipBox(SlipTab),
    Para(ParaTab),
    Project(i64),
    More,
    NotFound,
}

/// The tab of the tab bar a route belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Today,
    Notes,
    SlipBox,
    Para,
    More,
}

impl Route {
    /// Read a `location.hash`: `#/notes/1a`, `#/slipbox/tree`, `#/notes?kind=hub`.
    #[must_use]
    pub fn parse(hash: &str) -> Self {
        let raw = hash.trim_start_matches('#');
        let (path, query) = raw.split_once('?').unwrap_or((raw, ""));
        let parts: Vec<String> = path
            .split('/')
            .filter(|p| !p.is_empty())
            .map(decode)
            .collect();
        let parts: Vec<&str> = parts.iter().map(String::as_str).collect();
        match parts.as_slice() {
            [] => Self::Today,
            ["notes"] => Self::Notes(param(query, "kind")),
            ["notes", zid] => Self::Note((*zid).to_string()),
            ["slipbox"] => Self::SlipBox(SlipTab::Inbox),
            ["slipbox", tab] => SlipTab::ALL
                .into_iter()
                .find(|t| t.slug() == *tab)
                .map_or(Self::NotFound, Self::SlipBox),
            ["para"] => Self::Para(ParaTab::Projects),
            ["para", tab] => ParaTab::ALL
                .into_iter()
                .find(|t| t.slug() == *tab)
                .map_or(Self::NotFound, Self::Para),
            ["projects", id] => id.parse().map_or(Self::NotFound, Self::Project),
            ["more"] => Self::More,
            _ => Self::NotFound,
        }
    }

    /// The hash that leads here.
    #[must_use]
    pub fn href(&self) -> String {
        match self {
            Self::Today | Self::NotFound => "#/".into(),
            Self::Notes(None) => "#/notes".into(),
            Self::Notes(Some(kind)) => format!("#/notes?kind={}", encode(kind)),
            Self::Note(zid) => format!("#/notes/{}", encode(zid)),
            Self::SlipBox(tab) => format!("#/slipbox/{}", tab.slug()),
            Self::Para(tab) => format!("#/para/{}", tab.slug()),
            Self::Project(id) => format!("#/projects/{id}"),
            Self::More => "#/more".into(),
        }
    }

    #[must_use]
    pub const fn tab(&self) -> Tab {
        match self {
            Self::Today | Self::NotFound => Tab::Today,
            Self::Notes(_) | Self::Note(_) => Tab::Notes,
            Self::SlipBox(_) => Tab::SlipBox,
            Self::Para(_) | Self::Project(_) => Tab::Para,
            Self::More => Tab::More,
        }
    }
}

fn param(query: &str, key: &str) -> Option<String> {
    query
        .split('&')
        .filter_map(|kv| kv.split_once('='))
        .find(|(k, _)| *k == key)
        .map(|(_, v)| decode(v))
        .filter(|v| !v.is_empty())
}

/// Percent-encode everything but unreserved characters, as `encodeURIComponent`.
#[must_use]
pub fn encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) {
            out.push(char::from(b));
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

/// Undo [`encode`]; a malformed escape is kept as written.
#[must_use]
pub fn decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes
            .get(i + 1..i + 3)
            .and_then(|h| std::str::from_utf8(h).ok());
        match (bytes[i], hex.and_then(|h| u8::from_str_radix(h, 16).ok())) {
            (b'%', Some(b)) => {
                out.push(b);
                i += 3;
            }
            (b'+', _) => {
                out.push(b' ');
                i += 1;
            }
            (b, _) => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_screen() {
        assert_eq!(Route::parse(""), Route::Today);
        assert_eq!(Route::parse("#/"), Route::Today);
        assert_eq!(Route::parse("#/notes"), Route::Notes(None));
        assert_eq!(
            Route::parse("#/notes?kind=hub"),
            Route::Notes(Some("hub".into()))
        );
        assert_eq!(Route::parse("#/notes/1a"), Route::Note("1a".into()));
        assert_eq!(Route::parse("#/slipbox"), Route::SlipBox(SlipTab::Inbox));
        assert_eq!(
            Route::parse("#/slipbox/tree"),
            Route::SlipBox(SlipTab::Tree)
        );
        assert_eq!(Route::parse("#/para/areas"), Route::Para(ParaTab::Areas));
        assert_eq!(Route::parse("#/projects/7"), Route::Project(7));
        assert_eq!(Route::parse("#/more"), Route::More);
        assert_eq!(Route::parse("#/projects/x"), Route::NotFound);
        assert_eq!(Route::parse("#/slipbox/nope"), Route::NotFound);
    }

    #[test]
    fn href_round_trips() {
        for r in [
            Route::Today,
            Route::Notes(None),
            Route::Notes(Some("permanent".into())),
            Route::Note("1a/2 b".into()),
            Route::SlipBox(SlipTab::Tags),
            Route::Para(ParaTab::Tasks),
            Route::Project(12),
            Route::More,
        ] {
            assert_eq!(Route::parse(&r.href()), r);
        }
    }

    #[test]
    fn tabs_follow_sections() {
        assert_eq!(Route::Note("1".into()).tab(), Tab::Notes);
        assert_eq!(Route::Project(1).tab(), Tab::Para);
    }

    #[test]
    fn decode_keeps_malformed_escapes() {
        assert_eq!(decode("a%2Fb%zz%"), "a/b%zz%");
        assert_eq!(decode("%C3%A9"), "é");
    }
}
