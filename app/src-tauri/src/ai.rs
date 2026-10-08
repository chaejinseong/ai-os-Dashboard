use crate::model::Item;
use chrono::{DateTime, Datelike, Local, TimeZone, Utc};
use serde_json::{json, Value};

const API: &str = "https://api.anthropic.com/v1/messages";
const SYSTEM: &str = "너는 사용자의 받은편지함을 돕는 비서다. 메일이 사용자의 답장이나 조치를 필요로 하는지 판단한다. 뉴스레터, 광고, 자동 알림, 단순 공지는 조치 불필요다. JSON 한 줄로만 답한다: {\"needs_action\": true|false, \"summary\": \"40자 이내 한국어 한 줄 요약\"}";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Judgment {
    pub needs_action: bool,
    pub summary: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ResultWithUsage {
    pub judgment: Judgment,
    pub input_tokens: u64,
    pub output_tokens: u64,
}
#[derive(Debug, thiserror::Error)]
pub enum JudgeError {
    #[error("AI 요청 실패 ({0})")]
    Status(u16),
    #[error("AI 네트워크 또는 응답 오류")]
    Network,
}

pub const INPUT_OVERHEAD_TOKENS: u64 = 64;

pub fn request_body(model: &str, item: &Item) -> Value {
    let snippet: String = item.meta["snippet"]
        .as_str()
        .unwrap_or_default()
        .chars()
        .take(500)
        .collect();
    json!({
        "model": model,
        "max_tokens": 150,
        "temperature": 0,
        "system": SYSTEM,
        "messages": [{"role": "user", "content": format!(
            "보낸 사람: {}\n제목: {}\n본문 앞부분: {}",
            item.subtitle.as_deref().unwrap_or(""), item.title, snippet
        )}]
    })
}

pub fn build_request(
    client: &reqwest::Client,
    endpoint: &str,
    key: &str,
    body: &Value,
) -> Result<reqwest::Request, String> {
    client
        .post(endpoint)
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(body)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|_| "AI 요청 생성 실패".into())
}

pub fn parse_response(value: &Value) -> ResultWithUsage {
    let text = value["content"][0]["text"].as_str().unwrap_or_default();
    let judgment = text
        .find('{')
        .zip(text.rfind('}'))
        .and_then(|(start, end)| text.get(start..=end))
        .and_then(|candidate| serde_json::from_str::<Value>(candidate).ok())
        .and_then(|data| {
            let needs_action = data["needs_action"].as_bool()?;
            let summary = data["summary"]
                .as_str()
                .map(|s| s.chars().take(40).collect());
            Some(Judgment {
                needs_action,
                summary,
            })
        })
        .unwrap_or_else(|| {
            eprintln!("AI 응답 JSON 파싱 실패");
            Judgment {
                needs_action: true,
                summary: None,
            }
        });
    ResultWithUsage {
        judgment,
        input_tokens: value["usage"]["input_tokens"].as_u64().unwrap_or(0),
        output_tokens: value["usage"]["output_tokens"].as_u64().unwrap_or(0),
    }
}

pub fn cost_usd(input_tokens: u64, output_tokens: u64, price_in: f64, price_out: f64) -> f64 {
    ((input_tokens as f64) * price_in + (output_tokens as f64) * price_out) / 1_000_000.0
}
pub fn worst_case_cost(body: &Value, price_in: f64, price_out: f64) -> f64 {
    let input = body["system"].as_str().unwrap_or_default().len() as u64
        + body["messages"][0]["content"]
            .as_str()
            .unwrap_or_default()
            .len() as u64
        + INPUT_OVERHEAD_TOKENS;
    let output = body["max_tokens"].as_u64().unwrap_or(0);
    cost_usd(input, output, price_in, price_out)
}
pub fn fits_cap(spent: f64, reserved: f64, worst: f64, cap: f64) -> bool {
    spent + reserved + worst <= cap
}

pub fn month_start_utc(now: DateTime<Local>) -> DateTime<Utc> {
    Local
        .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
        .earliest()
        .expect("local month start")
        .with_timezone(&Utc)
}

pub async fn judge(
    client: &reqwest::Client,
    key: &str,
    model: &str,
    item: &Item,
) -> Result<ResultWithUsage, JudgeError> {
    let request = build_request(client, API, key, &request_body(model, item))
        .map_err(|_| JudgeError::Network)?;
    let response = client
        .execute(request)
        .await
        .map_err(|_| JudgeError::Network)?;
    if !response.status().is_success() {
        return Err(JudgeError::Status(response.status().as_u16()));
    }
    let value = response
        .json::<Value>()
        .await
        .map_err(|_| JudgeError::Network)?;
    Ok(parse_response(&value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{fixture, ItemKind};
    use chrono::{Duration, Timelike};

    #[test]
    fn request_body_and_headers() {
        let mut item = fixture("subject", ItemKind::Email, Utc::now());
        item.subtitle = Some("Alice".into());
        item.meta["snippet"] = "가".repeat(510).into();
        let body = request_body("test-model", &item);
        assert_eq!(body["model"], "test-model");
        assert_eq!(body["max_tokens"], 150);
        assert_eq!(body["temperature"], 0);
        let content = body["messages"][0]["content"].as_str().unwrap();
        assert!(content.starts_with("보낸 사람: Alice\n제목: subject\n본문 앞부분: "));
        assert_eq!(
            content
                .split("본문 앞부분: ")
                .nth(1)
                .unwrap()
                .chars()
                .count(),
            500
        );
        let request = build_request(
            &reqwest::Client::new(),
            "https://example.test/messages",
            "fake-key",
            &body,
        )
        .unwrap();
        assert_eq!(request.method(), reqwest::Method::POST);
        assert_eq!(request.headers()["x-api-key"], "fake-key");
        assert_eq!(request.headers()["anthropic-version"], "2023-06-01");
        assert_eq!(request.headers()["content-type"], "application/json");
    }
    #[test]
    fn parses_json_and_safe_fallback() {
        let wrap = |text: &str| json!({"content":[{"text":text}], "usage":{"input_tokens":100,"output_tokens":10}});
        let normal = parse_response(&wrap(r#"{"needs_action":false,"summary":"공지"}"#));
        assert!(!normal.judgment.needs_action);
        assert_eq!(normal.judgment.summary.as_deref(), Some("공지"));
        let padded = parse_response(&wrap(
            "설명 {\"needs_action\":true,\"summary\":\"답장 필요\"} 끝",
        ));
        assert!(padded.judgment.needs_action);
        assert_eq!(padded.judgment.summary.as_deref(), Some("답장 필요"));
        let broken = parse_response(&wrap("broken {json}"));
        assert!(broken.judgment.needs_action);
        assert!(broken.judgment.summary.is_none());
        assert_eq!(broken.input_tokens, 100);
        assert_eq!(broken.output_tokens, 10);
    }
    #[test]
    fn cost_and_local_month_boundary() {
        assert!((cost_usd(1_000_000, 2_000_000, 1.0, 5.0) - 11.0).abs() < f64::EPSILON);
        let now = Local.with_ymd_and_hms(2026, 10, 8, 12, 0, 0).unwrap();
        let start = month_start_utc(now);
        assert_eq!(start.with_timezone(&Local).date_naive().day(), 1);
        assert_eq!(start.with_timezone(&Local).hour(), 0);
        assert!(now.with_timezone(&Utc) - start >= Duration::days(7));
    }
    #[test]
    fn worst_case_uses_utf8_bytes_and_cap_boundary() {
        let mut item = fixture("", ItemKind::Email, Utc::now());
        item.meta["snippet"] = "가나".into();
        let body = request_body("test", &item);
        let system_bytes = body["system"].as_str().unwrap().len() as u64;
        let content = body["messages"][0]["content"].as_str().unwrap();
        assert!(content.ends_with("가나"));
        let content_bytes = content.len() as u64;
        assert_eq!(
            content_bytes - content.trim_end_matches("가나").len() as u64,
            6
        );
        let expected = cost_usd(system_bytes + content_bytes + 64, 150, 1.0, 5.0);
        assert!((worst_case_cost(&body, 1.0, 5.0) - expected).abs() < f64::EPSILON);
        assert!(fits_cap(0.25, 0.25, 0.5, 1.0));
        assert!(!fits_cap(0.25, 0.25, 0.5 + 1e-12, 1.0));
    }
}
