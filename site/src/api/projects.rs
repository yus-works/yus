use actix_web::{get, web};
use moka::future::Cache;
use octocrab::Octocrab;
use once_cell::sync::Lazy;
use once_cell::sync::OnceCell;
use serde::Serialize;
use std::{env, sync::Arc, time::Duration};

use crate::languages::LANG_TABLE;

#[derive(Serialize, Clone)]
struct LangDto {
    name: String,
    pct: f32,
    color: Option<String>,
    icon: Option<String>,
}

#[derive(Serialize, Clone)]
struct ProjectDto {
    name: String,
    description: Option<String>,
    version: Option<String>,
    status: String,
    labels: Vec<String>, // from repo topics
    languages: Vec<LangDto>,

    repo_url: String,
}

static CLIENT: OnceCell<Arc<Octocrab>> = OnceCell::new();

/// 5-minute cache keyed by login string
static PROJECT_CACHE: Lazy<Cache<String, Arc<Vec<ProjectDto>>>> = Lazy::new(|| {
    Cache::builder()
        .time_to_live(Duration::from_secs(300)) // 5 min
        .max_capacity(32) // safety cap
        .build()
});

fn gh() -> &'static Octocrab {
    CLIENT
        .get_or_init(|| {
            let token = env::var("PROJECTS_PAT").expect("missing PAT");
            Octocrab::builder()
                .personal_token(token)
                .build()
                .expect("client")
                .into()
        })
        .as_ref()
}

#[get("/api/projects")]
async fn projects() -> actix_web::Result<web::Json<Vec<ProjectDto>>> {
    let login = "yus-works".to_string();

    if let Some(cached) = PROJECT_CACHE.get(&login).await {
        return Ok(web::Json((*cached).clone()));
    }

    // 1. run query
    let data: serde_json::Value = gh()
        .graphql(&serde_json::json!({
            "query":     include_str!("repos.graphql"),
            "variables": { "login": login }
        }))
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // 2. brab the repo array
    let repos = data
        .pointer("/data/repositoryOwner/repos/nodes")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            eprintln!("Unexpected GraphQL shape: {:#?}", data);
            actix_web::error::ErrorInternalServerError("repos missing")
        })?;

    // 3. current time -> derive “status”
    let now = chrono::Utc::now();

    // 4. build the DTOs
    let out: Vec<ProjectDto> = repos
        .iter()
        .filter_map(|repo| {
            // name is the only field we require
            let name = repo.get("name")?.as_str()?.to_owned();

            // optional bits
            let description = repo
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from);

            // tag = first element in refs.nodes
            let version = repo
                .pointer("/refs/nodes")
                .and_then(|v| v.get(0))
                .and_then(|v| v.get("name"))
                .and_then(|v| v.as_str())
                .map(String::from);

            // pushedAt -> months -> status
            let pushed_at = repo.get("pushedAt")?.as_str()?;
            let pushed: chrono::DateTime<chrono::Utc> = pushed_at.parse().ok()?;
            let months = (now - pushed).num_days() / 30;
            let status = match months {
                m if m <= 1 => "active",
                m if m <= 6 => "ongoing",
                m if m <= 18 => "paused",
                _ => "done / archived",
            }
            .to_string();

            // topics
            let labels = repo
                .pointer("/repositoryTopics/nodes")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|t| {
                            t.pointer("/topic/name")
                                .and_then(|n| n.as_str())
                                .map(String::from)
                        })
                        .collect()
                })
                .unwrap_or_default();

            let langs_json: Vec<serde_json::Value> = repo
                .pointer("/languages/edges")
                .and_then(|v| v.as_array())
                .cloned() // turn &Vec<_> into Vec<_>
                .unwrap_or_default(); // empty vec if field missing

            let total: f32 = langs_json
                .iter()
                .filter_map(|l| l.get("size").and_then(|s| s.as_u64()))
                .map(|s| s as f32)
                .sum();

            let languages = if total == 0.0 {
                Vec::new()
            } else {
                langs_json
                    .iter()
                    .filter_map(|l| {
                        let name = l.pointer("/node/name")?.as_str()?.to_owned();
                        let pct = (l.get("size")?.as_u64()? as f32 / total) * 100.0;

                        let meta = LANG_TABLE.get(&name);
                        Some(LangDto {
                            name,
                            pct,
                            color: meta.and_then(|m| m.color.as_ref().cloned()),
                            icon: meta.and_then(|m| m.devicon.as_ref().cloned()),
                        })
                    })
                    .collect()
            };

            let repo_url = repo.get("url")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_owned();

            Some(ProjectDto {
                name,
                description,
                version,
                status,
                labels,
                languages,

                repo_url,
            })
        })
        .collect();

    PROJECT_CACHE.insert(login, Arc::new(out.clone())).await;
    Ok(web::Json(out))
}
