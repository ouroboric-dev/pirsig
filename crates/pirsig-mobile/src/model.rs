//! The parts of Pirsig's API answers the phone shows. The `zettelkasten`
//! crate's DTOs (`src/dtos/`) are the contract; these read the fields used here
//! and ignore the rest.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total_pages: u64,
    pub total_items: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Note {
    pub zid: String,
    pub title: String,
    #[serde(default)]
    pub body: String,
    pub kind: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Link {
    pub zid: String,
    pub title: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct NoteDetail {
    /// The note's own fields sit beside its links in the JSON.
    #[serde(flatten)]
    pub note: Note,
    pub links: Vec<Link>,
    pub backlinks: Vec<Link>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TreeEntry {
    pub zid: String,
    pub title: String,
    pub kind: String,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TagCount {
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Stats {
    pub notes: u64,
    pub links: u64,
    pub inbox: u64,
    pub orphans: u64,
    pub tags: Vec<TagCount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Readiness {
    pub ready: usize,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub status: Option<String>,
    pub area_id: i64,
    pub due_on: Option<String>,
    pub archived_at: Option<String>,
}

impl Project {
    #[must_use]
    pub fn status(&self) -> &str {
        self.status.as_deref().unwrap_or("active")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ProjectNote {
    pub zid: String,
    pub title: String,
    pub kind: String,
    pub context: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub done: bool,
    pub due_on: Option<String>,
    pub project_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Area {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Resource {
    pub id: i64,
    pub title: String,
    pub kind: Option<String>,
    pub url: Option<String>,
    pub area_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Hippo {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Feed {
    pub id: i64,
}

/// `POST /api/notes`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NewNote {
    pub title: String,
    pub body: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hippo_id: Option<i64>,
}

/// `POST /api/projects/{id}/notes`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PullNote {
    pub zid: String,
}

/// `PATCH /api/tasks/{id}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TaskDone {
    pub done: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_note_detail_as_the_api_sends_it() {
        let json = r#"{"id":1,"zid":"1","title":"Atomic notes","body":"x","kind":"permanent",
            "source":null,"resource_id":null,"hippo_id":null,"tags":["pkm"],
            "created_at":"2026-10-26T09:00:00Z","updated_at":"2026-10-26T09:00:00Z",
            "links":[{"zid":"1a","title":"One idea","kind":"permanent","context":null,"auto":false}],
            "backlinks":[]}"#;
        let d: NoteDetail = serde_json::from_str(json).unwrap();
        assert_eq!(d.note.tags, vec!["pkm"]);
        assert_eq!(d.links[0].zid, "1a");
    }
}
