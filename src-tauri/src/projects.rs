use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tauri::{State, Window};

use crate::security;
use crate::settings::SharedSettings;

const GITHUB_API: &str = "https://api.github.com";
const CACHE_TTL: Duration = Duration::from_secs(300);
const STALE_DAYS: i64 = 180;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCheck {
    status: &'static str,
    detail: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    name: String,
    full_name: String,
    description: Option<String>,
    url: String,
    is_private: bool,
    is_archived: bool,
    pushed_at: Option<String>,
    open_issues_count: u64,
    open_prs_count: usize,
    releases_count: usize,
    health_score: u8,
    checks: HashMap<&'static str, ProjectCheck>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectsSnapshot {
    projects: Vec<Project>,
    average_health: u8,
    attention_count: usize,
    fetched_at: i64,
}

struct CachedSnapshot {
    created: Instant,
    snapshot: ProjectsSnapshot,
}

static CACHE: OnceLock<Mutex<Option<CachedSnapshot>>> = OnceLock::new();

#[derive(Clone, Deserialize)]
struct GithubRepo {
    name: String,
    full_name: String,
    description: Option<String>,
    html_url: String,
    private: bool,
    archived: bool,
    pushed_at: Option<String>,
    open_issues_count: u64,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    published_at: Option<String>,
    created_at: String,
}

fn check(status: &'static str, detail: impl Into<String>) -> ProjectCheck {
    ProjectCheck {
        status,
        detail: detail.into(),
    }
}

async fn github_get(client: &Client, token: &str, path: &str) -> Result<reqwest::Response, String> {
    client
        .get(format!("{GITHUB_API}{path}"))
        .bearer_auth(token)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .timeout(Duration::from_secs(12))
        .send()
        .await
        .map_err(|e| format!("GitHub request failed: {e}"))
}

async fn exists(client: &Client, token: &str, path: &str) -> bool {
    github_get(client, token, path)
        .await
        .map(|response| response.status().is_success())
        .unwrap_or(false)
}

async fn count_items(client: &Client, token: &str, path: &str) -> usize {
    match github_get(client, token, path).await {
        Ok(response) if response.status().is_success() => response
            .json::<Vec<serde_json::Value>>()
            .await
            .map(|items| items.len())
            .unwrap_or(0),
        _ => 0,
    }
}

async fn inspect_repo(client: Client, token: String, repo: GithubRepo) -> Project {
    if repo.archived {
        let checks = [
            ("readme", check("na", "Archived")),
            ("releases", check("na", "Archived")),
            ("unreleased", check("na", "Archived")),
            ("roadmap", check("na", "Archived")),
            ("branding", check("na", "Archived")),
            ("setup", check("na", "Archived")),
            ("activity", check("na", "Archived")),
            ("openActivity", check("na", "Archived")),
        ]
        .into_iter()
        .collect();
        return project_from(repo, checks, 100, 0, 0);
    }

    let base = format!("/repos/{}", repo.full_name);
    let readme_path = format!("{base}/readme");
    let roadmap_path = format!("{base}/contents/ROADMAP.md");
    let install_path = format!("{base}/contents/INSTALL.md");
    let contributing_path = format!("{base}/contents/CONTRIBUTING.md");
    let github_path = format!("{base}/contents/.github");
    let docs_path = format!("{base}/contents/docs");
    let assets_path = format!("{base}/contents/assets");
    let milestones_path = format!("{base}/milestones?state=open&per_page=1");
    let pulls_path = format!("{base}/pulls?state=open&per_page=100");
    let releases_path = format!("{base}/releases?per_page=100");
    let (
        has_readme,
        has_roadmap,
        has_install,
        has_contributing,
        has_github,
        has_docs,
        has_assets,
        milestones,
        open_prs,
        releases_count,
        release,
    ) = tokio::join!(
        exists(&client, &token, &readme_path),
        exists(&client, &token, &roadmap_path),
        exists(&client, &token, &install_path),
        exists(&client, &token, &contributing_path),
        exists(&client, &token, &github_path),
        exists(&client, &token, &docs_path),
        exists(&client, &token, &assets_path),
        count_items(&client, &token, &milestones_path),
        count_items(&client, &token, &pulls_path),
        count_items(&client, &token, &releases_path),
        async {
            match github_get(&client, &token, &format!("{base}/releases/latest")).await {
                Ok(response) if response.status().is_success() => {
                    response.json::<GithubRelease>().await.ok()
                }
                _ => None,
            }
        }
    );

    let mut checks = HashMap::new();
    checks.insert(
        "readme",
        if has_readme {
            check("pass", "README found")
        } else {
            check("fail", "No README found")
        },
    );
    checks.insert(
        "releases",
        release
            .as_ref()
            .map(|item| check("pass", format!("Latest: {}", item.tag_name)))
            .unwrap_or_else(|| check("warn", "No published releases")),
    );

    let unreleased = if let Some(latest) = &release {
        let since = latest.published_at.as_ref().unwrap_or(&latest.created_at);
        let path = format!(
            "{base}/commits?since={}&per_page=1",
            urlencoding::encode(since)
        );
        if count_items(&client, &token, &path).await > 0 {
            check("warn", format!("Commits since {}", latest.tag_name))
        } else {
            check("pass", "Up to date with latest release")
        }
    } else {
        check("na", "No releases to compare")
    };
    checks.insert("unreleased", unreleased);
    checks.insert(
        "roadmap",
        if has_roadmap {
            check("pass", "ROADMAP.md found")
        } else if milestones > 0 {
            check("pass", "Open milestone found")
        } else {
            check("warn", "No roadmap or open milestone")
        },
    );
    checks.insert(
        "branding",
        if has_github || has_docs || has_assets {
            check("pass", "Branding directory found")
        } else {
            check("warn", "No .github, docs, or assets directory")
        },
    );
    checks.insert(
        "setup",
        if has_install || has_contributing {
            check("pass", "Setup guide found")
        } else {
            check("warn", "No INSTALL.md or CONTRIBUTING.md")
        },
    );
    checks.insert(
        "activity",
        match repo.pushed_at.as_deref().and_then(parse_github_time) {
            Some(timestamp) if unix_now() - timestamp > STALE_DAYS * 86_400 => {
                check("warn", "No push activity in 6 months")
            }
            Some(_) => check("pass", "Active in last 6 months"),
            None => check("warn", "No push activity recorded"),
        },
    );
    let total_open = repo.open_issues_count as usize + open_prs;
    checks.insert(
        "openActivity",
        if total_open > 0 {
            check(
                "warn",
                format!("{} issue(s), {open_prs} PR(s)", repo.open_issues_count),
            )
        } else {
            check("na", "Nothing open")
        },
    );

    let score = health_score(&checks);
    project_from(repo, checks, score, open_prs, releases_count)
}

fn project_from(
    repo: GithubRepo,
    checks: HashMap<&'static str, ProjectCheck>,
    health_score: u8,
    open_prs_count: usize,
    releases_count: usize,
) -> Project {
    Project {
        name: repo.name,
        full_name: repo.full_name,
        description: repo.description,
        url: repo.html_url,
        is_private: repo.private,
        is_archived: repo.archived,
        pushed_at: repo.pushed_at,
        open_issues_count: repo.open_issues_count,
        open_prs_count,
        releases_count,
        health_score,
        checks,
    }
}

fn health_score(checks: &HashMap<&str, ProjectCheck>) -> u8 {
    let weights = [
        ("readme", 3.0),
        ("releases", 2.0),
        ("unreleased", 1.0),
        ("roadmap", 1.0),
        ("branding", 1.0),
        ("setup", 1.0),
        ("activity", 2.0),
    ];
    let mut total = 0.0_f64;
    let mut maximum = 0.0_f64;
    for (key, weight) in weights {
        let Some(item) = checks.get(key) else {
            continue;
        };
        if item.status == "na" {
            continue;
        }
        maximum += weight;
        total += match item.status {
            "pass" => weight,
            "warn" => weight * 0.5,
            _ => 0.0,
        };
    }
    if maximum == 0.0 {
        100
    } else {
        ((total / maximum) * 100.0).round() as u8
    }
}

fn parse_github_time(value: &str) -> Option<i64> {
    let date = value.get(..10)?;
    let mut parts = date.split('-');
    let year = parts.next()?.parse::<i64>().ok()?;
    let month = parts.next()?.parse::<i64>().ok()?;
    let day = parts.next()?.parse::<i64>().ok()?;
    Some(days_from_civil(year, month, day) * 86_400)
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let adjusted_month = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * adjusted_month + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

async fn fetch_snapshot(client: Client, token: String) -> Result<ProjectsSnapshot, String> {
    let response = github_get(
        &client,
        &token,
        "/user/repos?visibility=all&affiliation=owner&sort=pushed&per_page=100",
    )
    .await?;
    if response.status() == StatusCode::UNAUTHORIZED {
        return Err("GitHub token was rejected".to_string());
    }
    if !response.status().is_success() {
        return Err(format!("GitHub returned {}", response.status()));
    }
    let repos = response
        .json::<Vec<GithubRepo>>()
        .await
        .map_err(|e| format!("Invalid GitHub response: {e}"))?;

    let mut projects = Vec::with_capacity(repos.len());
    for batch in repos.chunks(5) {
        let mut tasks = tokio::task::JoinSet::new();
        for repo in batch.iter().cloned() {
            tasks.spawn(inspect_repo(client.clone(), token.clone(), repo));
        }
        while let Some(result) = tasks.join_next().await {
            projects.push(result.map_err(|e| e.to_string())?);
        }
    }
    projects.sort_by_key(|project| {
        (
            project.health_score,
            std::cmp::Reverse(project.pushed_at.clone()),
        )
    });
    let active: Vec<_> = projects
        .iter()
        .filter(|project| !project.is_archived)
        .collect();
    let average_health = if active.is_empty() {
        100
    } else {
        (active
            .iter()
            .map(|project| project.health_score as usize)
            .sum::<usize>()
            / active.len()) as u8
    };
    let attention_count = active
        .iter()
        .filter(|project| project.health_score < 80)
        .count();
    Ok(ProjectsSnapshot {
        projects,
        average_health,
        attention_count,
        fetched_at: unix_now(),
    })
}

#[tauri::command]
pub async fn get_projects(
    refresh: Option<bool>,
    window: Window,
    settings: State<'_, SharedSettings>,
    http: State<'_, crate::http::HttpClient>,
) -> Result<Option<ProjectsSnapshot>, String> {
    security::require_window(&window, &["main", "demo-projects"])?;
    let token = settings
        .lock()
        .map_err(|e| e.to_string())?
        .github_token
        .trim()
        .to_string();
    if token.is_empty() {
        return Ok(None);
    }
    let cache = CACHE.get_or_init(|| Mutex::new(None));
    if refresh != Some(true) {
        if let Some(cached) = cache.lock().map_err(|e| e.to_string())?.as_ref() {
            if cached.created.elapsed() < CACHE_TTL {
                return Ok(Some(cached.snapshot.clone()));
            }
        }
    }
    let snapshot = fetch_snapshot(http.0.clone(), token).await?;
    *cache.lock().map_err(|e| e.to_string())? = Some(CachedSnapshot {
        created: Instant::now(),
        snapshot: snapshot.clone(),
    });
    Ok(Some(snapshot))
}

#[tauri::command]
pub fn open_project_url(url: String, window: Window) -> Result<(), String> {
    security::require_window(&window, &["main"])?;
    if !url.starts_with("https://github.com/") {
        return Err("Only GitHub URLs are allowed".to_string());
    }
    open::that(url).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::{check, health_score};
    use std::collections::HashMap;

    #[test]
    fn projctrl_weights_are_preserved() {
        let checks = [
            ("readme", check("pass", "")),
            ("releases", check("warn", "")),
            ("unreleased", check("pass", "")),
            ("roadmap", check("pass", "")),
            ("branding", check("pass", "")),
            ("setup", check("pass", "")),
            ("activity", check("pass", "")),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();
        assert_eq!(health_score(&checks), 91);
    }
}
