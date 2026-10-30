use loco_rs::{testing::prelude::*, TestServer};
use serde_json::{json, Value};
use serial_test::serial;
use zettelkasten::app::App;

#[tokio::test]
#[serial]
async fn can_crud_areas_projects_tasks() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request
            .post("/api/areas")
            .json(&json!({ "name": "Health" }))
            .await;
        assert_eq!(res.status_code(), 201);
        let area: Value = res.json();
        let area_id = area["id"].as_i64().unwrap();

        let res = request
            .post("/api/projects")
            .json(&json!({ "name": "Run a marathon", "area_id": area_id, "status": "active" }))
            .await;
        assert_eq!(res.status_code(), 201);
        let project_id = res.json::<Value>()["id"].as_i64().unwrap();

        let res = request
            .post("/api/tasks")
            .json(
                &json!({ "title": "Buy shoes", "project_id": project_id, "due_on": "2026-11-01" }),
            )
            .await;
        assert_eq!(res.status_code(), 201);
        let task: Value = res.json();
        assert_eq!(task["done"], false);
        let task_id = task["id"].as_i64().unwrap();

        // partial update only touches the given fields
        let res = request
            .patch(&format!("/api/tasks/{task_id}"))
            .json(&json!({ "done": true }))
            .await;
        assert_eq!(res.status_code(), 200);
        let task: Value = res.json();
        assert_eq!(task["done"], true);
        assert_eq!(task["title"], "Buy shoes");

        let res = request
            .get(&format!("/api/tasks?project_id={project_id}&done=false"))
            .await;
        assert_eq!(res.json::<Value>()["total_items"], 0);

        let res = request
            .put(&format!("/api/areas/{area_id}"))
            .json(&json!({ "name": "Fitness" }))
            .await;
        assert_eq!(res.json::<Value>()["name"], "Fitness");

        let res = request.delete(&format!("/api/tasks/{task_id}")).await;
        assert_eq!(res.status_code(), 204);
        let res = request.get(&format!("/api/tasks/{task_id}")).await;
        assert_eq!(res.status_code(), 404);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn rejects_children_of_missing_parents() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request
            .post("/api/projects")
            .json(&json!({ "name": "Orphan", "area_id": 999 }))
            .await;
        assert_eq!(res.status_code(), 404);

        let res = request
            .post("/api/tasks")
            .json(&json!({ "title": "Orphan", "project_id": 999 }))
            .await;
        assert_eq!(res.status_code(), 404);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn archive_hides_from_default_listing() {
    request::<App, _, _>(|request, _ctx| async move {
        let area_id = request
            .post("/api/areas")
            .json(&json!({ "name": "Old hobby" }))
            .await
            .json::<Value>()["id"]
            .as_i64()
            .unwrap();
        let resource_id = request
            .post("/api/resources")
            .json(&json!({ "title": "Notes", "area_id": area_id }))
            .await
            .json::<Value>()["id"]
            .as_i64()
            .unwrap();

        let res = request.post(&format!("/api/areas/{area_id}/archive")).await;
        assert!(!res.json::<Value>()["archived_at"].is_null());
        request
            .post(&format!("/api/resources/{resource_id}/archive"))
            .await;

        let active: Value = request.get("/api/areas").await.json();
        assert_eq!(active["total_items"], 0);
        let archived: Value = request.get("/api/areas?archived=true").await.json();
        assert_eq!(archived["total_items"], 1);
        let resources: Value = request.get("/api/resources?archived=true").await.json();
        assert_eq!(resources["items"][0]["title"], "Notes");

        let res = request
            .post(&format!("/api/areas/{area_id}/unarchive"))
            .await;
        assert!(res.json::<Value>()["archived_at"].is_null());
        let active: Value = request.get("/api/areas").await.json();
        assert_eq!(active["total_items"], 1);
    })
    .await;
}

async fn post_id(request: &TestServer, path: &str, body: Value) -> i64 {
    let res = request.post(path).json(&body).await;
    assert_eq!(res.status_code(), 201, "{path}: {}", res.text());
    res.json::<Value>()["id"].as_i64().unwrap()
}

#[tokio::test]
#[serial]
async fn names_are_trimmed_unique_and_never_blank() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request
            .post("/api/areas")
            .json(&json!({ "name": "  Weather  " }))
            .await;
        assert_eq!(res.json::<Value>()["name"], "Weather");
        let res = request
            .post("/api/areas")
            .json(&json!({ "name": "weather" }))
            .await;
        assert_eq!(res.status_code(), 400, "same area twice, other case");
        let res = request
            .post("/api/areas")
            .json(&json!({ "name": " " }))
            .await;
        assert_eq!(res.status_code(), 400);

        let area = post_id(&request, "/api/areas", json!({ "name": "Health" })).await;
        let res = request
            .post("/api/projects")
            .json(&json!({ "name": "", "area_id": area }))
            .await;
        assert_eq!(res.status_code(), 400);
        let res = request
            .post("/api/projects")
            .json(&json!({ "name": "Marathon", "area_id": area, "status": "bogus" }))
            .await;
        assert_eq!(res.status_code(), 400);
        let res = request
            .post("/api/projects")
            .json(&json!({ "name": "Marathon", "area_id": area, "due_on": "2027-04-18" }))
            .await;
        let project: Value = res.json();
        assert_eq!(project["status"], "active", "status defaults to active");
        assert_eq!(project["due_on"], "2027-04-18");
        let res = request
            .post("/api/tasks")
            .json(&json!({ "title": "\n", "project_id": project["id"] }))
            .await;
        assert_eq!(res.status_code(), 400);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn null_clears_and_absent_keeps() {
    request::<App, _, _>(|request, _ctx| async move {
        let area = post_id(
            &request,
            "/api/areas",
            json!({ "name": "Health", "description": "body" }),
        )
        .await;
        let project = post_id(
            &request,
            "/api/projects",
            json!({ "name": "Marathon", "area_id": area, "description": "42k", "due_on": "2027-04-18" }),
        )
        .await;
        let res = request
            .patch(&format!("/api/projects/{project}"))
            .json(&json!({ "due_on": null }))
            .await;
        let p: Value = res.json();
        assert!(p["due_on"].is_null());
        assert_eq!(p["description"], "42k", "absent field is kept");

        let res = request
            .patch(&format!("/api/areas/{area}"))
            .json(&json!({ "description": null }))
            .await;
        assert!(res.json::<Value>()["description"].is_null());

        let task = post_id(
            &request,
            "/api/tasks",
            json!({ "title": "Shoes", "project_id": project, "due_on": "2026-11-01", "notes": "size 44" }),
        )
        .await;
        let res = request
            .patch(&format!("/api/tasks/{task}"))
            .json(&json!({ "due_on": null, "notes": null }))
            .await;
        let t: Value = res.json();
        assert!(t["due_on"].is_null() && t["notes"].is_null());

        let resource = post_id(
            &request,
            "/api/resources",
            json!({ "title": "Plan", "url": "https://example.com/plan", "area_id": area, "project_id": project }),
        )
        .await;
        let res = request
            .patch(&format!("/api/resources/{resource}"))
            .json(&json!({ "url": null, "project_id": null }))
            .await;
        let r: Value = res.json();
        assert!(r["url"].is_null() && r["project_id"].is_null());
        assert_eq!(r["area_id"], area);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn deleting_a_parent_needs_force() {
    request::<App, _, _>(|request, _ctx| async move {
        let area = post_id(&request, "/api/areas", json!({ "name": "Health" })).await;
        let project = post_id(
            &request,
            "/api/projects",
            json!({ "name": "Marathon", "area_id": area }),
        )
        .await;
        post_id(
            &request,
            "/api/tasks",
            json!({ "title": "Shoes", "project_id": project }),
        )
        .await;
        let resource = post_id(
            &request,
            "/api/resources",
            json!({ "title": "Couch to 5k", "area_id": area }),
        )
        .await;

        let res = request.delete(&format!("/api/projects/{project}")).await;
        assert_eq!(res.status_code(), 400, "project still has a task");
        let res = request.delete(&format!("/api/areas/{area}")).await;
        assert_eq!(res.status_code(), 400, "area still has a project");
        assert_eq!(
            request
                .get(&format!("/api/projects/{project}"))
                .await
                .status_code(),
            200
        );

        let res = request
            .delete(&format!("/api/areas/{area}?force=true"))
            .await;
        assert_eq!(res.status_code(), 204);
        assert_eq!(
            request
                .get(&format!("/api/projects/{project}"))
                .await
                .status_code(),
            404
        );
        let tasks: Value = request.get("/api/tasks").await.json();
        assert_eq!(tasks["total_items"], 0, "tasks went with their project");
        let r: Value = request
            .get(&format!("/api/resources/{resource}"))
            .await
            .json();
        assert!(r["area_id"].is_null(), "the resource stays, unfiled");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn nothing_active_lives_in_the_archive() {
    request::<App, _, _>(|request, _ctx| async move {
        let area = post_id(&request, "/api/areas", json!({ "name": "Old hobby" })).await;
        let project = post_id(
            &request,
            "/api/projects",
            json!({ "name": "Chess club", "area_id": area }),
        )
        .await;
        post_id(
            &request,
            "/api/resources",
            json!({ "title": "Openings", "area_id": area }),
        )
        .await;

        request.post(&format!("/api/areas/{area}/archive")).await;
        let p: Value = request
            .get(&format!("/api/projects/{project}"))
            .await
            .json();
        assert!(
            !p["archived_at"].is_null(),
            "archiving the area archives its projects"
        );
        let resources: Value = request.get("/api/resources").await.json();
        assert_eq!(resources["total_items"], 0, "and its resources");

        let res = request
            .post("/api/projects")
            .json(&json!({ "name": "New", "area_id": area }))
            .await;
        assert_eq!(res.status_code(), 400, "no new project in an archived area");
        let res = request
            .post(&format!("/api/projects/{project}/unarchive"))
            .await;
        assert_eq!(
            res.status_code(),
            400,
            "no project back while its area is archived"
        );
        let res = request
            .post("/api/tasks")
            .json(&json!({ "title": "Play", "project_id": project }))
            .await;
        assert_eq!(res.status_code(), 400, "no new task in an archived project");

        request.post(&format!("/api/areas/{area}/unarchive")).await;
        let res = request
            .post(&format!("/api/projects/{project}/unarchive"))
            .await;
        assert_eq!(res.status_code(), 200);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn resources_are_well_formed() {
    request::<App, _, _>(|request, _ctx| async move {
        let health = post_id(&request, "/api/areas", json!({ "name": "Health" })).await;
        let money = post_id(&request, "/api/areas", json!({ "name": "Money" })).await;
        let project = post_id(
            &request,
            "/api/projects",
            json!({ "name": "Marathon", "area_id": health }),
        )
        .await;
        for bad in [
            json!({ "title": "x", "url": "not a url" }),
            json!({ "title": "x", "kind": "podcast-ish" }),
            json!({ "title": "x", "citekey": "has space" }),
            json!({ "title": "x", "area_id": money, "project_id": project }),
        ] {
            let res = request.post("/api/resources").json(&bad).await;
            assert_eq!(res.status_code(), 400, "{bad}");
        }
        post_id(
            &request,
            "/api/resources",
            json!({ "title": "Attention", "kind": "Paper", "citekey": "@vaswani2017attention" }),
        )
        .await;
        let res = request
            .post("/api/resources")
            .json(&json!({ "title": "Again", "citekey": "vaswani2017attention" }))
            .await;
        assert_eq!(res.status_code(), 400, "citekeys are unique");

        // moving the project moves the resources filed under it and its area
        let r = post_id(
            &request,
            "/api/resources",
            json!({ "title": "Plan", "area_id": health, "project_id": project }),
        )
        .await;
        request
            .patch(&format!("/api/projects/{project}"))
            .json(&json!({ "area_id": money }))
            .await;
        let r: Value = request.get(&format!("/api/resources/{r}")).await.json();
        assert_eq!(r["area_id"], money);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn projects_pull_notes_in_without_owning_them() {
    request::<App, _, _>(|request, _ctx| async move {
        let area = post_id(&request, "/api/areas", json!({ "name": "Weather" })).await;
        let project = post_id(
            &request,
            "/api/projects",
            json!({ "name": "Buoy bench", "area_id": area }),
        )
        .await;
        request
            .post("/api/notes")
            .json(
                &json!({ "title": "Score against buoys", "kind": "permanent", "tags": ["buoys"] }),
            )
            .await;
        let res = request
            .post(&format!("/api/projects/{project}/notes"))
            .json(&json!({ "zid": "1", "context": "the method" }))
            .await;
        let pulled: Value = res.json();
        assert_eq!(pulled[0]["zid"], "1");
        assert_eq!(pulled[0]["context"], "the method");
        assert_eq!(pulled[0]["tags"], json!(["buoys"]));
        let res = request
            .post(&format!("/api/projects/{project}/notes"))
            .json(&json!({ "zid": "404" }))
            .await;
        assert_eq!(res.status_code(), 404);

        request
            .delete(&format!("/api/projects/{project}?force=true"))
            .await;
        assert_eq!(request.get("/api/notes/1").await.status_code(), 200);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn readiness_reports_each_method() {
    request::<App, _, _>(|request, _ctx| async move {
        let empty: Value = request.get("/api/readiness").await.json();
        let names: Vec<&str> = empty["sections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            names,
            [
                "PARA",
                "Zettelkasten",
                "Obsidian in/out",
                "RSS in/out",
                "arXiv in",
                "Hippo rhizome",
                "Plasticity"
            ]
        );
        assert_eq!(empty["sections"][0]["checks"][0]["status"], "missing");
        assert!(empty["sections"][0]["checks"][0]["fix"].is_string());

        let area = post_id(&request, "/api/areas", json!({ "name": "Weather" })).await;
        let project = post_id(
            &request,
            "/api/projects",
            json!({ "name": "Buoy bench", "area_id": area, "status": "done" }),
        )
        .await;
        request
            .post("/api/notes")
            .json(&json!({ "title": "Lonely claim", "kind": "permanent" }))
            .await;
        let r: Value = request.get("/api/readiness").await.json();
        let para = &r["sections"][0];
        assert_eq!(para["checks"][0]["status"], "ready");
        let done = para["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == "done projects archived")
            .unwrap();
        assert_eq!(done["status"], "partial");
        assert!(done["detail"]
            .as_str()
            .unwrap()
            .contains(&format!("#{project} Buoy bench")));
        let zk = &r["sections"][1];
        let orphans = zk["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == "no orphan permanent notes")
            .unwrap();
        assert_eq!(orphans["status"], "partial");
        assert!(r["ready"].as_u64().unwrap() < r["total"].as_u64().unwrap());
    })
    .await;
}
