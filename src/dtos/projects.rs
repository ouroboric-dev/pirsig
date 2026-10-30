use loco_rs::model::ModelResult;
use sea_orm::prelude::{Date, DateTimeWithTimeZone};
use sea_orm::ConnectionTrait;
use ts_rs::TS;

use crate::{dtos::common::Patch, models::notes};

#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct ProjectDto {
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    /// `active`, `waiting`, `someday` or `done`.
    pub status: Option<String>,
    #[ts(type = "number")]
    pub area_id: i64,
    /// The deadline that makes it a project and not an area.
    #[ts(type = "string | null")]
    pub due_on: Option<Date>,
    #[ts(type = "string | null")]
    pub archived_at: Option<DateTimeWithTimeZone>,
    #[ts(type = "string")]
    pub created_at: DateTimeWithTimeZone,
    #[ts(type = "string")]
    pub updated_at: DateTimeWithTimeZone,
}

impl From<crate::models::_entities::projects::Model> for ProjectDto {
    fn from(m: crate::models::_entities::projects::Model) -> Self {
        Self {
            id: m.id,
            name: m.name,
            description: m.description,
            status: m.status,
            area_id: m.area_id,
            due_on: m.due_on,
            archived_at: m.archived_at,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct CreateProject {
    pub name: String,
    pub description: Option<String>,
    /// Defaults to `active`.
    pub status: Option<String>,
    #[ts(type = "number")]
    pub area_id: i64,
    #[ts(type = "string | null")]
    pub due_on: Option<Date>,
}

/// Partial update: only the fields that are present are changed; `null` clears
/// an optional field.
#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct UpdateProject {
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Patch::is_keep")]
    #[ts(type = "string | null")]
    pub description: Patch<String>,
    pub status: Option<String>,
    #[ts(type = "number | null")]
    pub area_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Patch::is_keep")]
    #[ts(type = "string | null")]
    pub due_on: Patch<Date>,
}

/// A note a project pulls in, and why.
#[derive(serde::Serialize, serde::Deserialize, TS)]
pub struct ProjectNoteDto {
    pub zid: String,
    pub title: String,
    pub kind: String,
    pub context: Option<String>,
    pub tags: Vec<String>,
}

impl ProjectNoteDto {
    /// The project's notes with their tags, in one query for the tags.
    ///
    /// # Errors
    ///
    /// When the tags can't be read.
    pub async fn many<C: ConnectionTrait>(
        db: &C,
        notes: Vec<(notes::Model, Option<String>)>,
    ) -> ModelResult<Vec<Self>> {
        let ids: Vec<i64> = notes.iter().map(|(n, _)| n.id).collect();
        let mut tags = notes::Entity::tags_of(db, &ids).await?;
        Ok(notes
            .into_iter()
            .map(|(n, context)| Self {
                tags: tags.remove(&n.id).unwrap_or_default(),
                zid: n.zid,
                title: n.title,
                kind: n.kind,
                context,
            })
            .collect())
    }
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, TS)]
pub struct PullNote {
    /// zid of the note to pull in.
    pub zid: String,
    /// Why the note belongs to the project.
    pub context: Option<String>,
}
