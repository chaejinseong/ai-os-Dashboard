use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ItemKind {
    Email,
    Event,
    Task,
    PullRequest,
    Issue,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Stage {
    Requested,
    InProgress,
    Review,
    Done,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub source: String,
    pub kind: ItemKind,
    pub title: String,
    pub subtitle: Option<String>,
    pub url: String,
    pub timestamp: DateTime<Utc>,
    pub due: Option<DateTime<Utc>>,
    pub done: bool,
    pub stage: Option<Stage>,
    pub meta: serde_json::Value,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SourceStatus {
    pub source: String,
    pub connected: bool,
    pub syncing: bool,
    pub last_synced: Option<DateTime<Utc>>,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub sync_minutes: u32,
    pub mail_unanswered_days: u32,
    pub ai_model: String,
    pub ai_monthly_cap_usd: f64,
    pub ai_price_in_per_mtok: f64,
    pub ai_price_out_per_mtok: f64,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            sync_minutes: 15,
            mail_unanswered_days: 2,
            ai_model: "claude-haiku-4-5-20251001".into(),
            ai_monthly_cap_usd: 5.0,
            ai_price_in_per_mtok: 1.0,
            ai_price_out_per_mtok: 5.0,
        }
    }
}
impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=1440).contains(&self.sync_minutes)
            || !(1..=365).contains(&self.mail_unanswered_days)
        {
            return Err("동기화는 1~1440분, 미응답 기준은 1~365일로 입력하세요".into());
        }
        if self.ai_model.trim().is_empty()
            || ![
                self.ai_monthly_cap_usd,
                self.ai_price_in_per_mtok,
                self.ai_price_out_per_mtok,
            ]
            .iter()
            .all(|value| value.is_finite() && *value >= 0.0)
        {
            Err("AI 모델을 입력하고 상한·가격은 0 이상의 숫자로 입력하세요".into())
        } else {
            Ok(())
        }
    }
}
#[cfg(test)]
pub fn fixture(id: &str, kind: ItemKind, timestamp: DateTime<Utc>) -> Item {
    Item {
        id: id.into(),
        source: "test".into(),
        kind,
        title: id.into(),
        subtitle: None,
        url: "https://example.com".into(),
        timestamp,
        due: None,
        done: false,
        stage: None,
        meta: serde_json::json!({"last_from_me":false}),
    }
}
