use super::{Connector, ConnectorError};
use crate::{
    model::{Item, ItemKind, Stage},
    oauth::GoogleAuth,
};
use chrono::{DateTime, Duration, Local, NaiveDate, TimeZone, Utc};
use serde_json::Value;
use std::{collections::HashSet, sync::Arc};

pub struct GoogleCalendar {
    auth: Arc<GoogleAuth>,
    client: reqwest::Client,
}

impl GoogleCalendar {
    pub fn new(auth: Arc<GoogleAuth>) -> Self {
        Self {
            auth,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl Connector for GoogleCalendar {
    fn id(&self) -> &'static str {
        "calendar"
    }
    fn label(&self) -> &'static str {
        "Calendar"
    }
    async fn is_connected(&self) -> bool {
        self.auth.is_connected()
    }

    async fn fetch(&self) -> Result<Vec<Item>, ConnectorError> {
        let token = self.auth.access_token().await?;
        let now = Utc::now();
        let time_min = (now - Duration::days(1)).to_rfc3339();
        let time_max = (now + Duration::days(14)).to_rfc3339();
        let mut page_token = None;
        let mut seen_tokens = HashSet::new();
        let mut items = Vec::new();
        loop {
            let mut request = self
                .client
                .get("https://www.googleapis.com/calendar/v3/calendars/primary/events")
                .bearer_auth(&token)
                .query(&[
                    ("timeMin", time_min.as_str()),
                    ("timeMax", time_max.as_str()),
                    ("singleEvents", "true"),
                    ("orderBy", "startTime"),
                    ("maxResults", "250"),
                ])
                .timeout(std::time::Duration::from_secs(30));
            if let Some(page) = &page_token {
                request = request.query(&[("pageToken", page)]);
            }
            let response = request
                .send()
                .await
                .map_err(|_| ConnectorError::Message("일정 네트워크 연결 실패".into()))?;
            if !response.status().is_success() {
                return Err(ConnectorError::Message(if response.status() == 401 {
                    "다시 로그인 필요".into()
                } else {
                    format!("일정 조회 실패 ({})", response.status().as_u16())
                }));
            }
            let data: Value = response
                .json()
                .await
                .map_err(|_| ConnectorError::Message("일정 응답 형식 오류".into()))?;
            if let Some(events) = data.get("items") {
                let events = events
                    .as_array()
                    .ok_or_else(|| ConnectorError::Message("일정 목록 형식 오류".into()))?;
                items.extend(events.iter().filter_map(event_to_item));
            }
            match data
                .get("nextPageToken")
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                Some(next) if seen_tokens.insert(next.to_owned()) => {
                    page_token = Some(next.to_owned())
                }
                Some(_) => return Err(ConnectorError::Message("일정 페이지 토큰 반복".into())),
                None => return Ok(items),
            }
        }
    }
}

fn event_time(value: &Value) -> Option<DateTime<Utc>> {
    if let Some(time) = value.get("dateTime").and_then(Value::as_str) {
        return DateTime::parse_from_rfc3339(time)
            .ok()
            .map(|t| t.with_timezone(&Utc));
    }
    let date = NaiveDate::parse_from_str(value.get("date")?.as_str()?, "%Y-%m-%d").ok()?;
    Local
        .from_local_datetime(&date.and_hms_opt(0, 0, 0)?)
        .earliest()
        .map(|t| t.with_timezone(&Utc))
}

pub fn event_to_item(json: &Value) -> Option<Item> {
    if json["status"].as_str() == Some("cancelled") {
        return None;
    }
    let my_response = json["attendees"]
        .as_array()
        .and_then(|attendees| attendees.iter().find(|a| a["self"].as_bool() == Some(true)))
        .and_then(|a| a["responseStatus"].as_str());
    Some(Item {
        id: json["id"].as_str()?.into(),
        source: "calendar".into(),
        kind: ItemKind::Event,
        title: json["summary"].as_str().unwrap_or("(제목 없음)").into(),
        subtitle: json["location"].as_str().map(str::to_owned),
        url: json["htmlLink"].as_str()?.into(),
        timestamp: event_time(&json["start"])?,
        due: event_time(&json["end"]),
        done: false,
        stage: (my_response == Some("needsAction")).then_some(Stage::Requested),
        meta: serde_json::json!({
            "all_day": json["start"]["date"].is_string(),
            "my_response": my_response,
            "created": json["created"],
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn event() -> Value {
        serde_json::json!({"id":"event", "htmlLink":"https://calendar.google.com/event",
            "created":"2026-10-01T01:00:00Z", "start":{"dateTime":"2026-10-07T10:00:00+09:00"},
            "end":{"dateTime":"2026-10-07T11:00:00+09:00"}})
    }
    #[test]
    fn all_day_uses_local_midnight_and_exclusive_end() {
        let mut data = event();
        data["start"] = serde_json::json!({"date":"2026-10-07"});
        data["end"] = serde_json::json!({"date":"2026-10-08"});
        let item = event_to_item(&data).unwrap();
        assert_eq!(
            item.timestamp,
            Local
                .with_ymd_and_hms(2026, 10, 7, 0, 0, 0)
                .unwrap()
                .with_timezone(&Utc)
        );
        assert_eq!(
            item.due,
            Some(
                Local
                    .with_ymd_and_hms(2026, 10, 8, 0, 0, 0)
                    .unwrap()
                    .with_timezone(&Utc)
            )
        );
        assert_eq!(item.meta["all_day"], true);
    }
    #[test]
    fn cancelled_is_excluded() {
        let mut data = event();
        data["status"] = "cancelled".into();
        assert!(event_to_item(&data).is_none());
    }
    #[test]
    fn no_attendees_and_missing_subject() {
        let item = event_to_item(&event()).unwrap();
        assert_eq!(item.title, "(제목 없음)");
        assert!(item.stage.is_none());
        assert!(item.meta["my_response"].is_null());
        assert_eq!(item.meta["all_day"], false);
        assert_eq!(item.timestamp.to_rfc3339(), "2026-10-07T01:00:00+00:00");
    }
    #[test]
    fn only_self_response_sets_requested_stage() {
        let mut data = event();
        data["attendees"] = serde_json::json!([
            {"responseStatus":"accepted"}, {"self":true,"responseStatus":"needsAction"}]);
        let item = event_to_item(&data).unwrap();
        assert_eq!(item.stage, Some(Stage::Requested));
        assert_eq!(item.meta["my_response"], "needsAction");
        assert_eq!(item.meta["created"], "2026-10-01T01:00:00Z");
        data["attendees"] = serde_json::json!([
            {"responseStatus":"needsAction"}, {"self":true,"responseStatus":"accepted"}]);
        assert!(event_to_item(&data).unwrap().stage.is_none());
    }
}
