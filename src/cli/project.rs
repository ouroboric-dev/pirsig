use std::fmt::Write;

use clap::ArgGroup;
use loco_rs::{app::AppContext, Error, Result};
use sea_orm::prelude::Date;
use serde::Serialize;

use super::{archived_tag, explain, named, note::found, patch, required, Output};
use crate::{
    dtos::projects::{CreateProject, ProjectDto, ProjectNoteDto, UpdateProject},
    models::{notes, projects},
};

/// `psg project -c -n "Run a marathon" -a 1 --due 2027-04-18`, `psg project -u 2 --status done`,
/// `psg project --pull 2 --note 1a -w "the training plan"`, `psg project -d 2`.
// clap flags are bools by nature
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, clap::Args)]
#[command(group(ArgGroup::new("action").args(["create", "update", "delete", "show", "archive", "unarchive", "pull", "drop"])))]
pub struct Args {
    /// Create a project (needs --name and --area).
    #[arg(short, long)]
    create: bool,
    /// Update the project with this id.
    #[arg(short, long, value_name = "ID")]
    update: Option<i64>,
    /// Delete the project with this id (and, with --force, its tasks).
    #[arg(short, long, value_name = "ID")]
    delete: Option<i64>,
    /// Show one project with the notes it pulls in.
    #[arg(short, long, value_name = "ID")]
    show: Option<i64>,
    /// Move the project to the archive.
    #[arg(long, value_name = "ID")]
    archive: Option<i64>,
    /// Bring the project back from the archive.
    #[arg(long, value_name = "ID")]
    unarchive: Option<i64>,
    /// Pull a note (--note) into this project, saying why (--why).
    #[arg(long, value_name = "ID")]
    pull: Option<i64>,
    /// Drop a note (--note) from this project; the note stays in the slipbox.
    #[arg(long, value_name = "ID")]
    drop: Option<i64>,

    /// Project name.
    #[arg(short, long)]
    name: Option<String>,
    /// Longer description; `""` clears it.
    #[arg(long)]
    description: Option<String>,
    /// active (default), waiting, someday or done.
    #[arg(long)]
    status: Option<String>,
    /// Deadline, YYYY-MM-DD.
    #[arg(long, value_name = "DATE")]
    due: Option<Date>,
    /// Clear the deadline.
    #[arg(long, conflicts_with = "due")]
    no_due: bool,
    /// Area id: the project's parent, or the list filter.
    #[arg(short, long, value_name = "AREA_ID")]
    area: Option<i64>,
    /// The note to pull in or drop (zid).
    #[arg(long, value_name = "ZID")]
    note: Option<String>,
    /// Why the note belongs to the project.
    #[arg(short, long)]
    why: Option<String>,
    /// When listing, show archived projects instead.
    #[arg(long)]
    archived: bool,
    /// Delete even when the project still has tasks.
    #[arg(short, long)]
    force: bool,
}

fn line(p: &ProjectDto) -> String {
    let status = p
        .status
        .as_deref()
        .map(|s| format!(" ({s})"))
        .unwrap_or_default();
    let due = p.due_on.map(|d| format!(" due {d}")).unwrap_or_default();
    format!(
        "#{} {}{status}{due} [area #{}]{}",
        p.id,
        p.name,
        p.area_id,
        archived_tag(p.archived_at.is_some())
    )
}

/// A project with the notes it pulls in.
#[derive(Serialize)]
struct ProjectDetail {
    #[serde(flatten)]
    project: ProjectDto,
    notes: Vec<ProjectNoteDto>,
}

fn detail(d: &ProjectDetail) -> String {
    let mut s = line(&d.project);
    if let Some(desc) = &d.project.description {
        let _ = write!(s, "\n{desc}");
    }
    for n in &d.notes {
        let why = n
            .context
            .as_deref()
            .map(|c| format!(": {c}"))
            .unwrap_or_default();
        let _ = write!(s, "\n  ~ {} {}{why}", n.zid, n.title);
    }
    s
}

async fn show(ctx: &AppContext, project: projects::Model) -> Result<ProjectDetail> {
    let notes = ProjectNoteDto::many(&ctx.db, project.notes(&ctx.db).await?).await?;
    Ok(ProjectDetail {
        project: project.into(),
        notes,
    })
}

pub async fn run(ctx: &AppContext, out: &Output, args: Args) -> Result<()> {
    let db = &ctx.db;
    if args.create {
        let area_id = required(args.area, "--area")?;
        let params = CreateProject {
            name: required(args.name, "--name")?,
            description: args.description,
            status: args.status,
            area_id,
            due_on: args.due,
        };
        let item = named(
            projects::ActiveModel::create(db, params).await,
            "area",
            area_id,
        )?;
        out.one(&ProjectDto::from(item), line);
    } else if let Some(id) = args.update {
        let params = UpdateProject {
            name: args.name,
            description: args.description.into(),
            status: args.status,
            area_id: args.area,
            due_on: patch(args.due, args.no_due),
        };
        let item = named(projects::Model::find(db, id).await, "project", id)?;
        let item = named(
            item.update_with(db, params).await,
            "area",
            args.area.unwrap_or_default(),
        )?;
        out.one(&ProjectDto::from(item), line);
    } else if let Some(id) = args.delete {
        let item = named(projects::Model::find(db, id).await, "project", id)?;
        named(item.remove(db, args.force).await, "project", id)?;
        out.deleted("project", id);
    } else if let Some(id) = args.show {
        let item = named(projects::Model::find(db, id).await, "project", id)?;
        out.one(&show(ctx, item).await?, detail);
    } else if let Some(id) = args.archive.or(args.unarchive) {
        let item = named(projects::Model::find(db, id).await, "project", id)?;
        let item = named(
            item.set_archived(db, args.archive.is_some()).await,
            "project",
            id,
        )?;
        out.one(&ProjectDto::from(item), line);
    } else if let Some(id) = args.pull.or(args.drop) {
        let project = named(projects::Model::find(db, id).await, "project", id)?;
        let zid = required(args.note, "--note")?;
        let note = found(notes::Model::find(db, &zid).await, &zid)?;
        if args.pull.is_some() {
            project
                .link_note(db, &note, args.why)
                .await
                .map_err(explain)?;
        } else if !project.unlink_note(db, &note).await? {
            return Err(Error::Message(format!(
                "note {zid} is not in project #{id}"
            )));
        }
        out.one(&show(ctx, project).await?, detail);
    } else {
        let items = projects::Entity::listing(args.area, args.archived)
            .all(db)
            .await?;
        let items: Vec<ProjectDto> = items.into_iter().map(Into::into).collect();
        out.many(&items, line);
    }
    Ok(())
}
