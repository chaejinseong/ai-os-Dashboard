use super::{Connector, ConnectorError};
use crate::{
    model::{Item, ItemKind, Stage},
    secrets,
    store::Store,
};
use chrono::{DateTime, TimeZone, Utc};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

const API: &str = "https://api.github.com";
const ACCEPT: &str = "application/vnd.github+json";
const VERSION: &str = "2022-11-28";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Reviewer,
    Author,
    Issue,
}

impl Role {
    fn name(self) -> &'static str {
        match self {
            Self::Reviewer => "reviewer",
            Self::Author => "author",
            Self::Issue => "issue",
        }
    }
}

pub struct GitHub {
    store: Arc<Mutex<Store>>,
    client: reqwest::Client,
}

impl GitHub {
    pub fn new(store: Arc<Mutex<Store>>) -> Self {
        Self {
            store,
            client: reqwest::Client::new(),
        }
    }

    pub async fn validate_login(token: &str) -> Result<String, String> {
        let data = get(&reqwest::Client::new(), token, &format!("{API}/user"), &[])
            .await
            .map_err(|error| error.to_string())?;
        data["login"]
            .as_str()
            .filter(|login| !login.is_empty())
            .map(str::to_owned)
            .ok_or_else(|| "GitHub 로그인 이름을 읽을 수 없습니다".into())
    }
}

async fn get(
    client: &reqwest::Client,
    token: &str,
    url: &str,
    query: &[(&str, &str)],
) -> Result<Value, ConnectorError> {
    let response = client
        .get(url)
        .bearer_auth(token)
        .header("Accept", ACCEPT)
        .header("X-GitHub-Api-Version", VERSION)
        .header("User-Agent", "ai-os-dashboard")
        .query(query)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|_| ConnectorError::Message("GitHub 네트워크 연결 실패".into()))?;
    if !response.status().is_success() {
        return Err(ConnectorError::Message(match response.status().as_u16() {
            401 => "GitHub 토큰이 유효하지 않습니다".into(),
            403 => "GitHub 권한 또는 요청 한도를 확인하세요".into(),
            code => format!("GitHub 조회 실패 ({code})"),
        }));
    }
    response
        .json()
        .await
        .map_err(|_| ConnectorError::Message("GitHub 응답 형식 오류".into()))
}

fn repo_name(json: &Value) -> Option<String> {
    let url = json["repository_url"].as_str()?;
    let path = url.strip_prefix("https://api.github.com/repos/")?;
    let mut parts = path.split('/');
    Some(format!("{}/{}", parts.next()?, parts.next()?))
}

fn parse_time(value: &Value) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value.as_str()?)
        .ok()
        .map(|time| time.with_timezone(&Utc))
}

pub fn requested_at_from_events(events: &Value, login: &str) -> Option<DateTime<Utc>> {
    events
        .as_array()?
        .iter()
        .filter(|event| {
            event["event"].as_str() == Some("review_requested")
                && event["requested_reviewer"]["login"]
                    .as_str()
                    .is_some_and(|reviewer| reviewer.eq_ignore_ascii_case(login))
        })
        .filter_map(|event| parse_time(&event["created_at"]))
        .max()
}

pub fn search_item_to_item(json: &Value, role: Role, requested_at: Option<DateTime<Utc>>) -> Item {
    let repo = repo_name(json);
    let url = json["html_url"].as_str().unwrap_or_default();
    let timestamp = parse_time(&json["updated_at"])
        .or_else(|| parse_time(&json["created_at"]))
        .unwrap_or_else(|| Utc.timestamp_opt(0, 0).unwrap());
    let requested_at = if role == Role::Reviewer {
        requested_at.or_else(|| parse_time(&json["created_at"]))
    } else {
        None
    };
    Item {
        id: url.into(),
        source: "github".into(),
        kind: if role == Role::Issue {
            ItemKind::Issue
        } else {
            ItemKind::PullRequest
        },
        title: json["title"].as_str().unwrap_or("(제목 없음)").into(),
        subtitle: repo.clone(),
        url: url.into(),
        timestamp,
        due: if role == Role::Issue {
            parse_time(&json["milestone"]["due_on"])
        } else {
            None
        },
        done: false,
        stage: Some(match role {
            Role::Reviewer => Stage::Review,
            Role::Author => Stage::InProgress,
            Role::Issue => Stage::Requested,
        }),
        meta: serde_json::json!({"role":role.name(), "requested_at":requested_at, "repo":repo}),
    }
}

fn dedupe_prs(reviewed: Vec<Item>, authored: Vec<Item>) -> Vec<Item> {
    let mut by_url: HashMap<String, Item> = HashMap::new();
    for item in reviewed {
        by_url.insert(item.id.clone(), item);
    }
    for item in authored {
        by_url.entry(item.id.clone()).or_insert(item);
    }
    by_url.into_values().collect()
}

#[async_trait::async_trait]
impl Connector for GitHub {
    fn id(&self) -> &'static str {
        "github"
    }
    fn label(&self) -> &'static str {
        "GitHub"
    }
    async fn is_connected(&self) -> bool {
        let has_token = secrets::get("github.token").ok().flatten().is_some();
        let has_login = self
            .store
            .lock()
            .ok()
            .and_then(|store| store.github_login().ok().flatten())
            .is_some();
        has_token && has_login
    }
    async fn fetch(&self) -> Result<Vec<Item>, ConnectorError> {
        let token = secrets::get("github.token")?
            .ok_or_else(|| ConnectorError::Message("GitHub 연결이 필요합니다".into()))?;
        let login = self
            .store
            .lock()
            .map_err(|_| ConnectorError::Message("캐시 잠금 실패".into()))?
            .github_login()?
            .ok_or_else(|| ConnectorError::Message("GitHub 로그인 정보가 없습니다".into()))?;
        let mut lists = Vec::new();
        for (query, role) in [
            (
                format!("is:open is:pr archived:false review-requested:{login}"),
                Role::Reviewer,
            ),
            (
                format!("is:open is:pr archived:false author:{login}"),
                Role::Author,
            ),
            (
                format!("is:open is:issue archived:false assignee:{login}"),
                Role::Issue,
            ),
        ] {
            let data = get(
                &self.client,
                &token,
                &format!("{API}/search/issues"),
                &[("q", &query), ("per_page", "50")],
            )
            .await?;
            let values = data["items"]
                .as_array()
                .ok_or_else(|| ConnectorError::Message("GitHub 검색 결과 형식 오류".into()))?;
            lists.push((role, values.clone()));
        }
        let mut reviewed = Vec::new();
        let mut authored = Vec::new();
        let mut issues = Vec::new();
        for (role, values) in lists {
            for (index, value) in values.iter().enumerate() {
                if value["html_url"].as_str().is_none()
                    || parse_time(&value["updated_at"]).is_none()
                {
                    continue;
                }
                let requested_at = if role == Role::Reviewer {
                    let fallback = parse_time(&value["created_at"]);
                    if index < 20 {
                        match (repo_name(value), value["number"].as_u64()) {
                            (Some(repo), Some(number)) => {
                                let url = format!("{API}/repos/{repo}/issues/{number}/events");
                                let events =
                                    get(&self.client, &token, &url, &[("per_page", "100")]).await?;
                                requested_at_from_events(&events, &login).or(fallback)
                            }
                            _ => fallback,
                        }
                    } else {
                        fallback
                    }
                } else {
                    None
                };
                let item = search_item_to_item(value, role, requested_at);
                match role {
                    Role::Reviewer => reviewed.push(item),
                    Role::Author => authored.push(item),
                    Role::Issue => issues.push(item),
                }
            }
        }
        let mut items = dedupe_prs(reviewed, authored);
        items.extend(issues);
        items.sort_by_key(|item| std::cmp::Reverse(item.timestamp));
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn search_item() -> Value {
        serde_json::json!({"title":"Fix bug", "html_url":"https://github.com/octo/repo/pull/7",
            "repository_url":"https://api.github.com/repos/octo/repo", "number":7,
            "created_at":"2026-10-01T00:00:00Z", "updated_at":"2026-10-08T00:00:00Z"})
    }
    #[test]
    fn converts_roles_and_missing_milestone() {
        let value = search_item();
        let reviewed = search_item_to_item(&value, Role::Reviewer, None);
        assert_eq!(reviewed.kind, ItemKind::PullRequest);
        assert_eq!(reviewed.stage, Some(Stage::Review));
        assert_eq!(reviewed.subtitle.as_deref(), Some("octo/repo"));
        assert_eq!(reviewed.meta["requested_at"], "2026-10-01T00:00:00Z");
        let authored = search_item_to_item(&value, Role::Author, None);
        assert_eq!(authored.stage, Some(Stage::InProgress));
        assert!(authored.meta["requested_at"].is_null());
        let issue = search_item_to_item(&value, Role::Issue, None);
        assert_eq!(issue.kind, ItemKind::Issue);
        assert_eq!(issue.stage, Some(Stage::Requested));
        assert!(issue.due.is_none());
        let mut with_due = value;
        with_due["milestone"] = serde_json::json!({"due_on":"2026-10-09T00:00:00Z"});
        assert_eq!(
            search_item_to_item(&with_due, Role::Issue, None)
                .due
                .unwrap()
                .to_rfc3339(),
            "2026-10-09T00:00:00+00:00"
        );
    }
    #[test]
    fn reviewer_wins_duplicate_pr() {
        let value = search_item();
        let items = dedupe_prs(
            vec![search_item_to_item(&value, Role::Reviewer, None)],
            vec![search_item_to_item(&value, Role::Author, None)],
        );
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].meta["role"], "reviewer");
    }
    #[test]
    fn latest_matching_review_request() {
        let events = serde_json::json!([
            {"event":"review_requested", "requested_reviewer":{"login":"other"}, "created_at":"2026-10-07T00:00:00Z"},
            {"event":"review_requested", "requested_reviewer":{"login":"ME"}, "created_at":"2026-10-05T00:00:00Z"},
            {"event":"review_requested", "requested_reviewer":{"login":"me"}, "created_at":"2026-10-06T00:00:00Z"},
            {"event":"review_request_removed", "requested_reviewer":{"login":"me"}, "created_at":"2026-10-08T00:00:00Z"}
        ]);
        assert_eq!(
            requested_at_from_events(&events, "me")
                .unwrap()
                .to_rfc3339(),
            "2026-10-06T00:00:00+00:00"
        );
    }
}
