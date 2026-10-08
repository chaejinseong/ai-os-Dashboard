use super::{Connector, ConnectorError};
use crate::{
    model::{Item, ItemKind},
    oauth::GoogleAuth,
};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::sync::Arc;
pub struct Gmail {
    auth: Arc<GoogleAuth>,
    client: reqwest::Client,
}
impl Gmail {
    pub fn new(auth: Arc<GoogleAuth>) -> Self {
        Self {
            auth,
            client: reqwest::Client::new(),
        }
    }
}
async fn get(
    client: &reqwest::Client,
    token: &str,
    path: &str,
    query: &[(&str, &str)],
) -> Result<Value, ConnectorError> {
    let r = client
        .get(format!(
            "https://gmail.googleapis.com/gmail/v1/users/me/{path}"
        ))
        .bearer_auth(token)
        .query(query)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|_| ConnectorError::Message("네트워크 연결 실패".into()))?;
    if !r.status().is_success() {
        return Err(ConnectorError::Message(if r.status() == 401 {
            "다시 로그인 필요".into()
        } else {
            format!("메일 조회 실패 ({})", r.status().as_u16())
        }));
    }
    r.json()
        .await
        .map_err(|_| ConnectorError::Message("메일 응답 형식 오류".into()))
}
#[async_trait::async_trait]
impl Connector for Gmail {
    fn id(&self) -> &'static str {
        "gmail"
    }
    fn label(&self) -> &'static str {
        "Gmail"
    }
    async fn is_connected(&self) -> bool {
        self.auth.is_connected()
    }
    async fn fetch(&self) -> Result<Vec<Item>, ConnectorError> {
        let token = self.auth.access_token().await?;
        let email = match self.auth.email().await {
            Some(e) => e,
            None => {
                let p = get(&self.client, &token, "profile", &[]).await?;
                let e = p["emailAddress"]
                    .as_str()
                    .ok_or_else(|| ConnectorError::Message("메일 주소 없음".into()))?
                    .to_owned();
                self.auth.set_email(e.clone()).await;
                e
            }
        };
        let list = get(
            &self.client,
            &token,
            "threads",
            &[("q", "in:inbox newer_than:14d"), ("maxResults", "100")],
        )
        .await?;
        let ids: Vec<String> = list["threads"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v["id"].as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        let mut items = Vec::new();
        for batch in ids.chunks(5) {
            let mut tasks = tokio::task::JoinSet::new();
            for id in batch {
                let (client, token, email) = (self.client.clone(), token.clone(), email.clone());
                let path = format!("threads/{id}");
                tasks.spawn(async move {
                    let data = get(
                        &client,
                        &token,
                        &path,
                        &[
                            ("format", "metadata"),
                            ("metadataHeaders", "From"),
                            ("metadataHeaders", "Subject"),
                            ("metadataHeaders", "Date"),
                        ],
                    )
                    .await?;
                    Ok::<_, ConnectorError>(thread_to_item(&data, &email))
                });
            }
            let mut results = Vec::new();
            while let Some(result) = tasks.join_next().await {
                results.push(result.map_err(|_| ConnectorError::Message("메일 조회 중단".into()))?);
            }
            items.extend(collect_threads(results)?);
        }
        Ok(items)
    }
}
fn collect_threads(
    results: Vec<Result<Option<Item>, ConnectorError>>,
) -> Result<Vec<Item>, ConnectorError> {
    let mut items = Vec::new();
    for result in results {
        match result {
            Ok(Some(item)) => items.push(item),
            Ok(None) => eprintln!("메일 스레드 형식 오류 — 건너뜀"),
            Err(error) => return Err(error),
        }
    }
    Ok(items)
}
fn from_parts(raw: &str) -> (String, String) {
    if let Some((name, rest)) = raw.rsplit_once('<') {
        if let Some((address, _)) = rest.split_once('>') {
            let name = name.trim().trim_matches('"');
            let address = address.trim();
            return (
                if name.is_empty() { address } else { name }.into(),
                address.into(),
            );
        }
    }
    (raw.trim().into(), raw.trim().into())
}
pub fn thread_to_item(json: &Value, my_email: &str) -> Option<Item> {
    let id = json["id"].as_str()?;
    let messages = json["messages"].as_array()?;
    let last = messages.iter().max_by_key(|m| {
        m["internalDate"]
            .as_str()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(i64::MIN)
    })?;
    let header = |key: &str| {
        last["payload"]["headers"]
            .as_array()
            .and_then(|h| {
                h.iter().find(|v| {
                    v["name"]
                        .as_str()
                        .is_some_and(|n| n.eq_ignore_ascii_case(key))
                })
            })
            .and_then(|v| v["value"].as_str())
    };
    let (name, address) = from_parts(header("From").unwrap_or(""));
    Some(Item {
        id: id.into(),
        source: "gmail".into(),
        kind: ItemKind::Email,
        title: header("Subject")
            .filter(|s| !s.is_empty())
            .unwrap_or("(제목 없음)")
            .into(),
        subtitle: Some(name),
        url: format!("https://mail.google.com/mail/u/0/#inbox/{id}"),
        timestamp: DateTime::<Utc>::from_timestamp_millis(
            last["internalDate"].as_str()?.parse().ok()?,
        )?,
        due: None,
        done: false,
        stage: None,
        meta: serde_json::json!({"thread_id":id,"last_message_id":last["id"],"last_from_me":address.eq_ignore_ascii_case(my_email.trim()),"snippet":last["snippet"].as_str().unwrap_or("")}),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn missing_subject_and_named_sender_case() {
        let json = serde_json::json!({"id":"thread","messages":[{"id":"message","internalDate":"1720000000000","snippet":"hello","payload":{"headers":[{"name":"From","value":"\"My Name\" <ME@Example.COM>"}]}}]});
        let i = thread_to_item(&json, "me@example.com").unwrap();
        assert_eq!(i.title, "(제목 없음)");
        assert_eq!(i.subtitle.as_deref(), Some("My Name"));
        assert_eq!(i.meta["last_from_me"], true);
        assert!(i.stage.is_none());
        assert!(!i.done);
    }
    #[test]
    fn address_only_and_invalid() {
        assert_eq!(
            from_parts("a@example.com"),
            ("a@example.com".into(), "a@example.com".into())
        );
        assert!(thread_to_item(&serde_json::json!({}), "me").is_none());
    }
    #[test]
    fn collect_threads_skips_bad_format_but_propagates_fetch_errors() {
        let valid = serde_json::json!({"id":"one","messages":[{"id":"message","internalDate":"1720000000000","payload":{"headers":[]}}]});
        let first = thread_to_item(&valid, "me").unwrap();
        let mut second = first.clone();
        second.id = "two".into();
        let items = collect_threads(vec![Ok(Some(first)), Ok(None), Ok(Some(second))]).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "one");
        assert_eq!(items[1].id, "two");
        assert!(collect_threads(vec![
            Ok(Some(items[0].clone())),
            Err(ConnectorError::Message("fetch failed".into())),
        ])
        .is_err());
    }
}
