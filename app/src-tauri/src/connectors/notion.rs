use super::{Connector, ConnectorError};
use crate::{
    model::{Item, ItemKind, Stage},
    secrets,
    store::Store,
};
use chrono::{DateTime, Local, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

const API: &str = "https://api.notion.com/v1";
const VERSION: &str = "2025-09-03";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mapping {
    database_id: String,
    data_source_id: String,
    title: String,
    due: String,
    status: String,
    stages: HashMap<String, Stage>,
}

pub struct Notion {
    store: Arc<Mutex<Store>>,
    client: reqwest::Client,
}

impl Notion {
    pub fn new(store: Arc<Mutex<Store>>) -> Self {
        Self {
            store,
            client: reqwest::Client::new(),
        }
    }

    pub async fn validate_and_map(token: &str, database: &str) -> Result<Mapping, String> {
        let database_id = database_id(database)?;
        let client = reqwest::Client::new();
        let database_json = get(&client, token, &format!("{API}/databases/{database_id}")).await?;
        let data_source_id = database_json["data_sources"][0]["id"]
            .as_str()
            .ok_or("데이터베이스에 데이터 소스가 없습니다")?
            .to_owned();
        let source_json = get(
            &client,
            token,
            &format!("{API}/data_sources/{data_source_id}"),
        )
        .await?;
        mapping_from_source(&source_json, database_id, data_source_id)
    }
}

async fn get(client: &reqwest::Client, token: &str, url: &str) -> Result<Value, String> {
    let response = client
        .get(url)
        .bearer_auth(token)
        .header("Notion-Version", VERSION)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|_| "Notion 네트워크 연결 실패".to_owned())?;
    if !response.status().is_success() {
        return Err(match response.status().as_u16() {
            401 => "Notion 토큰이 유효하지 않습니다".into(),
            404 => "통합에 데이터베이스가 공유되지 않았습니다".into(),
            code => format!("Notion 연결 실패 ({code})"),
        });
    }
    response
        .json()
        .await
        .map_err(|_| "Notion 응답 형식 오류".into())
}

#[async_trait::async_trait]
impl Connector for Notion {
    fn id(&self) -> &'static str {
        "notion"
    }
    fn label(&self) -> &'static str {
        "Notion"
    }
    async fn is_connected(&self) -> bool {
        let has_token = secrets::get("notion.token").ok().flatten().is_some();
        let has_mapping = self
            .store
            .lock()
            .ok()
            .and_then(|store| store.notion_mapping().ok().flatten())
            .is_some();
        has_token && has_mapping
    }
    async fn fetch(&self) -> Result<Vec<Item>, ConnectorError> {
        let token = secrets::get("notion.token")?
            .ok_or_else(|| ConnectorError::Message("Notion 연결이 필요합니다".into()))?;
        let mapping_json = self
            .store
            .lock()
            .map_err(|_| ConnectorError::Message("캐시 잠금 실패".into()))?
            .notion_mapping()?
            .ok_or_else(|| ConnectorError::Message("Notion 매핑이 없습니다".into()))?;
        let mapping: Mapping = serde_json::from_str(&mapping_json)
            .map_err(|_| ConnectorError::Message("Notion 매핑 형식 오류".into()))?;
        let mut items = Vec::new();
        let mut fetched_rows = 0usize;
        let mut cursor: Option<String> = None;
        let mut seen = std::collections::HashSet::new();
        loop {
            if cursor.is_some() {
                tokio::time::sleep(std::time::Duration::from_millis(350)).await;
            }
            let mut body = json!({"page_size": 100.min(1000 - fetched_rows)});
            if let Some(value) = &cursor {
                body["start_cursor"] = value.clone().into();
            }
            let response = self
                .client
                .post(format!(
                    "{API}/data_sources/{}/query",
                    mapping.data_source_id
                ))
                .bearer_auth(&token)
                .header("Notion-Version", VERSION)
                .json(&body)
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .await
                .map_err(|_| ConnectorError::Message("Notion 네트워크 연결 실패".into()))?;
            if !response.status().is_success() {
                return Err(ConnectorError::Message(format!(
                    "Notion 작업 조회 실패 ({})",
                    response.status().as_u16()
                )));
            }
            let data: Value = response
                .json()
                .await
                .map_err(|_| ConnectorError::Message("Notion 응답 형식 오류".into()))?;
            let pages = data["results"]
                .as_array()
                .ok_or_else(|| ConnectorError::Message("Notion 작업 목록 형식 오류".into()))?;
            fetched_rows += pages.len();
            items.extend(pages.iter().filter_map(|page| page_to_item(page, &mapping)));
            if fetched_rows >= 1000 || data["has_more"].as_bool() == Some(false) {
                return Ok(items);
            }
            let next = data["next_cursor"]
                .as_str()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| ConnectorError::Message("Notion 다음 페이지 정보 없음".into()))?;
            if !seen.insert(next.to_owned()) {
                return Err(ConnectorError::Message("Notion 페이지 반복".into()));
            }
            cursor = Some(next.to_owned());
        }
    }
}

pub fn database_id(input: &str) -> Result<String, String> {
    let input = input.trim();
    let path = input.split('?').next().unwrap_or("").trim_end_matches('/');
    let segment = path.rsplit('/').next().unwrap_or(path);
    let chars: Vec<char> = segment.chars().collect();
    for end in (1..=chars.len()).rev() {
        let mut hex = String::new();
        for ch in chars[..end].iter().rev() {
            if ch.is_ascii_hexdigit() {
                hex.push(*ch);
            } else if *ch != '-' {
                break;
            }
            if hex.len() == 32 {
                return Ok(hex.chars().rev().collect::<String>().to_lowercase());
            }
        }
    }
    Err("Notion 데이터베이스 URL 또는 ID를 확인하세요".into())
}

pub fn mapping_from_source(
    source: &Value,
    database_id: String,
    data_source_id: String,
) -> Result<Mapping, String> {
    let props = source["properties"]
        .as_object()
        .ok_or("Notion 데이터 소스 속성을 읽을 수 없습니다")?;
    let find_type = |kind: &str| {
        props
            .iter()
            .find(|(_, value)| value["type"].as_str() == Some(kind))
            .map(|(name, _)| name.clone())
    };
    let title = find_type("title").ok_or("제목 속성이 없습니다")?;
    let due = props
        .get("마감일")
        .filter(|value| value["type"].as_str() == Some("date"))
        .map(|_| "마감일".to_owned())
        .or_else(|| find_type("date"))
        .ok_or("날짜 속성이 없습니다")?;
    let status = props
        .get("진행 상태")
        .filter(|value| value["type"].as_str() == Some("status"))
        .map(|_| "진행 상태".to_owned())
        .or_else(|| find_type("status"))
        .ok_or("상태 속성이 없습니다")?;
    let schema = &props[&status]["status"];
    let options = schema["options"].as_array().ok_or("상태 옵션이 없습니다")?;
    let names_by_id: HashMap<&str, &str> = options
        .iter()
        .filter_map(|option| Some((option["id"].as_str()?, option["name"].as_str()?)))
        .collect();
    let mut stages = HashMap::new();
    if let Some(groups) = schema["groups"].as_array() {
        for group in groups {
            let stage = match group["name"]
                .as_str()
                .unwrap_or("")
                .to_ascii_lowercase()
                .as_str()
            {
                "to-do" | "to do" => Some(Stage::Requested),
                "in progress" => Some(Stage::InProgress),
                "complete" => Some(Stage::Done),
                _ => None,
            };
            if let (Some(stage), Some(ids)) = (stage, group["option_ids"].as_array()) {
                for id in ids {
                    if let Some(id) = id.as_str().filter(|id| names_by_id.contains_key(*id)) {
                        stages.insert(id.to_owned(), stage.clone());
                    }
                }
            }
        }
    }
    for (name, stage) in [
        ("시작 전", Stage::Requested),
        ("진행 중", Stage::InProgress),
        ("완료", Stage::Done),
    ] {
        if let Some(id) = options.iter().find_map(|option| {
            (option["name"].as_str() == Some(name))
                .then(|| option["id"].as_str())
                .flatten()
        }) {
            stages.entry(id.to_owned()).or_insert(stage);
        }
    }
    Ok(Mapping {
        database_id,
        data_source_id,
        title,
        due,
        status,
        stages,
    })
}

fn due_time(date: &str) -> Option<DateTime<Utc>> {
    if date.len() == 10 {
        let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
        return Local
            .from_local_datetime(&date.and_hms_opt(23, 59, 59)?)
            .latest()
            .map(|value| value.with_timezone(&Utc));
    }
    DateTime::parse_from_rfc3339(date)
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

pub fn page_to_item(json: &Value, mapping: &Mapping) -> Option<Item> {
    let properties = &json["properties"];
    let title = properties[&mapping.title]["title"]
        .as_array()
        .map(|parts| {
            parts
                .iter()
                .filter_map(|part| part["plain_text"].as_str())
                .collect::<String>()
        })
        .unwrap_or_default();
    let status_value = &properties[&mapping.status]["status"];
    let status_name = status_value["name"].as_str();
    let stage = status_value["id"]
        .as_str()
        .and_then(|id| mapping.stages.get(id))
        .or_else(|| status_name.and_then(|name| mapping.stages.get(name)))
        .cloned();
    let due = properties[&mapping.due]["date"]["start"]
        .as_str()
        .and_then(due_time);
    Some(Item {
        id: json["id"].as_str()?.into(),
        source: "notion".into(),
        kind: ItemKind::Task,
        title: if title.is_empty() {
            "(제목 없음)".into()
        } else {
            title
        },
        subtitle: None,
        url: json["url"].as_str()?.into(),
        timestamp: DateTime::parse_from_rfc3339(json["last_edited_time"].as_str()?)
            .ok()?
            .with_timezone(&Utc),
        due,
        done: stage == Some(Stage::Done),
        stage,
        meta: json!({"status_name": status_name}),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn source() -> Value {
        json!({"properties":{
            "제목":{"type":"title"}, "마감일":{"type":"date"},
            "진행 상태":{"type":"status","status":{
                "options":[{"id":"a","name":"시작 전"},{"id":"b","name":"진행 중"},{"id":"c","name":"완료"}],
                "groups":[{"name":"To-do","option_ids":["a"]},{"name":"In progress","option_ids":["b"]},{"name":"Complete","option_ids":["c"]}]
            }}
        }})
    }
    fn mapping(source: &Value) -> Mapping {
        mapping_from_source(source, "db".into(), "ds".into()).unwrap()
    }
    fn page(status: &str, due: Value) -> Value {
        let id = match status {
            "시작 전" => "a",
            "진행 중" => "b",
            "완료" => "c",
            _ => "unknown",
        };
        json!({"id":"page", "url":"https://notion.so/page", "last_edited_time":"2026-10-07T01:00:00Z",
            "properties":{"제목":{"title":[]}, "마감일":{"date":due},
                "진행 상태":{"status":{"id":id,"name":status}}}})
    }
    #[test]
    fn mapping_uses_named_properties_and_groups() {
        let m = mapping(&source());
        assert_eq!(
            (m.title.as_str(), m.due.as_str(), m.status.as_str()),
            ("제목", "마감일", "진행 상태")
        );
        assert_eq!(m.stages.get("a"), Some(&Stage::Requested));
        assert_eq!(m.stages.get("b"), Some(&Stage::InProgress));
        assert_eq!(m.stages.get("c"), Some(&Stage::Done));
    }
    #[test]
    fn mapping_falls_back_to_option_names_and_requires_date() {
        let mut data = source();
        data["properties"]["진행 상태"]["status"]["groups"] = json!([]);
        let m = mapping(&data);
        assert_eq!(m.stages.get("c"), Some(&Stage::Done));
        data["properties"].as_object_mut().unwrap().remove("마감일");
        assert!(mapping_from_source(&data, "db".into(), "ds".into()).is_err());
    }
    #[test]
    fn page_conversion_handles_due_title_and_stages() {
        let m = mapping(&source());
        let mut requested = page("시작 전", json!({"start":"2026-10-07"}));
        let item = page_to_item(&requested, &m).unwrap();
        assert_eq!(item.title, "(제목 없음)");
        assert_eq!(item.stage, Some(Stage::Requested));
        assert_eq!(
            item.due.unwrap().with_timezone(&Local).time().to_string(),
            "23:59:59"
        );
        requested["properties"]["제목"]["title"] =
            json!([{"plain_text":"첫 "},{"plain_text":"작업"}]);
        requested["properties"]["마감일"]["date"] = Value::Null;
        let item = page_to_item(&requested, &m).unwrap();
        assert_eq!(item.title, "첫 작업");
        assert!(item.due.is_none());
        assert_eq!(
            page_to_item(&page("진행 중", Value::Null), &m)
                .unwrap()
                .stage,
            Some(Stage::InProgress)
        );
        let done = page_to_item(&page("완료", Value::Null), &m).unwrap();
        assert_eq!(done.stage, Some(Stage::Done));
        assert!(done.done);
    }
    #[test]
    fn option_id_survives_rename_and_old_name_mapping_still_works() {
        let mapping = mapping(&source());
        let mut renamed = page("진행 중", Value::Null);
        renamed["properties"]["진행 상태"]["status"]["name"] = "Working".into();
        assert_eq!(
            page_to_item(&renamed, &mapping).unwrap().stage,
            Some(Stage::InProgress)
        );
        let mut old_mapping = mapping;
        old_mapping.stages = HashMap::from([("진행 중".into(), Stage::InProgress)]);
        let original = page("진행 중", Value::Null);
        assert_eq!(
            page_to_item(&original, &old_mapping).unwrap().stage,
            Some(Stage::InProgress)
        );
    }
    #[test]
    fn database_url_and_ids() {
        let id = "0123456789abcdef0123456789abcdef";
        assert_eq!(database_id(id).unwrap(), id);
        assert_eq!(
            database_id("01234567-89ab-cdef-0123-456789abcdef").unwrap(),
            id
        );
        assert_eq!(
            database_id(&format!("https://www.notion.so/Project-{id}?v=abc")).unwrap(),
            id
        );
        assert!(database_id("https://www.notion.so/invalid").is_err());
    }
}
