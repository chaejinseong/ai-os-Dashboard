use crate::{
    ai::{self, Judgment, ResultWithUsage},
    model::{Item, Settings, SourceStatus},
};
use chrono::{DateTime, Duration, Local, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;

pub struct Store {
    conn: Connection,
}
#[derive(Serialize)]
pub struct FeedbackStats {
    pub up: u64,
    pub down: u64,
    pub ratio: f64,
}
#[derive(Serialize)]
pub struct Stats {
    pub month_cost_usd: f64,
    pub month_input_tokens: u64,
    pub month_output_tokens: u64,
    pub cap_usd: f64,
    pub feedback_14d: FeedbackStats,
}
impl Store {
    pub fn open(path: &Path) -> Result<Self, String> {
        Self::from_connection(Connection::open(path).map_err(|e| e.to_string())?)
    }
    fn from_connection(conn: Connection) -> Result<Self, String> {
        conn.execute_batch("CREATE TABLE IF NOT EXISTS items(source TEXT NOT NULL,id TEXT NOT NULL,json TEXT NOT NULL,PRIMARY KEY(source,id)); CREATE TABLE IF NOT EXISTS source_status(source TEXT PRIMARY KEY,json TEXT NOT NULL); CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY,value TEXT NOT NULL); CREATE TABLE IF NOT EXISTS ai_judgments(thread_id TEXT NOT NULL,last_message_id TEXT NOT NULL,needs_action INTEGER NOT NULL,summary TEXT,created_at INTEGER NOT NULL,PRIMARY KEY(thread_id,last_message_id)); CREATE TABLE IF NOT EXISTS ai_usage(id INTEGER PRIMARY KEY,model TEXT NOT NULL,input_tokens INTEGER NOT NULL,output_tokens INTEGER NOT NULL,cost_usd REAL NOT NULL,created_at INTEGER NOT NULL); CREATE TABLE IF NOT EXISTS feedback(source TEXT NOT NULL,item_id TEXT NOT NULL,vote INTEGER NOT NULL,created_at INTEGER NOT NULL,PRIMARY KEY(source,item_id)); INSERT OR IGNORE INTO settings VALUES('sync_minutes','15'),('mail_unanswered_days','2'),('github_review_days','1'),('ai_model','claude-haiku-4-5-20251001'),('ai_monthly_cap_usd','5'),('ai_price_in_per_mtok','1'),('ai_price_out_per_mtok','5');").map_err(|e|e.to_string())?;
        Ok(Self { conn })
    }
    pub fn replace_items(&mut self, source: &str, items: &[Item]) -> Result<(), String> {
        if items.iter().any(|i| i.source != source) {
            return Err("캐시 소스 불일치".into());
        }
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM items WHERE source=?1", [source])
            .map_err(|e| e.to_string())?;
        for item in items {
            tx.execute(
                "INSERT INTO items VALUES(?1,?2,?3)",
                params![
                    source,
                    item.id,
                    serde_json::to_string(item).map_err(|e| e.to_string())?
                ],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())
    }
    fn read_json<T: serde::de::DeserializeOwned>(&self, table: &str) -> Result<Vec<T>, String> {
        let mut stmt = self
            .conn
            .prepare(&format!("SELECT json FROM {table}"))
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.map(|r| {
            serde_json::from_str(&r.map_err(|e| e.to_string())?).map_err(|e| e.to_string())
        })
        .collect()
    }
    pub fn items(&self) -> Result<Vec<Item>, String> {
        self.read_json("items")
    }
    pub fn statuses(&self) -> Result<Vec<SourceStatus>, String> {
        self.read_json("source_status")
    }
    pub fn save_status(&self, status: &SourceStatus) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO source_status VALUES(?1,?2)",
                params![
                    status.source,
                    serde_json::to_string(status).map_err(|e| e.to_string())?
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn notion_mapping(&self) -> Result<Option<String>, String> {
        match self.conn.query_row(
            "SELECT value FROM settings WHERE key='notion.mapping'",
            [],
            |row| row.get(0),
        ) {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }
    pub fn save_notion_mapping(&self, mapping: &str) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO settings(key,value) VALUES('notion.mapping',?1)",
                [mapping],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }
    pub fn clear_notion_mapping(&self) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM settings WHERE key='notion.mapping'", [])
            .map_err(|error| error.to_string())?;
        Ok(())
    }
    pub fn github_login(&self) -> Result<Option<String>, String> {
        match self.conn.query_row(
            "SELECT value FROM settings WHERE key='github.login'",
            [],
            |row| row.get(0),
        ) {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error.to_string()),
        }
    }
    pub fn save_github_login(&self, login: &str) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO settings(key,value) VALUES('github.login',?1)",
                [login],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }
    pub fn clear_github_login(&self) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM settings WHERE key='github.login'", [])
            .map_err(|error| error.to_string())?;
        Ok(())
    }
    pub fn github_review_days(&self) -> Result<u32, String> {
        self.conn
            .query_row(
                "SELECT value FROM settings WHERE key='github_review_days'",
                [],
                |row| row.get::<_, String>(0),
            )
            .map_err(|error| error.to_string())?
            .parse::<u32>()
            .map_err(|error| error.to_string())
    }
    pub fn settings(&self) -> Result<Settings, String> {
        let read = |key| {
            self.conn
                .query_row("SELECT value FROM settings WHERE key=?1", [key], |r| {
                    r.get::<_, String>(0)
                })
                .map_err(|e| e.to_string())?
                .parse::<u32>()
                .map_err(|e| e.to_string())
        };
        Ok(Settings {
            sync_minutes: read("sync_minutes")?,
            mail_unanswered_days: read("mail_unanswered_days")?,
            ai_model: self.setting("ai_model")?,
            ai_monthly_cap_usd: self
                .setting("ai_monthly_cap_usd")?
                .parse::<f64>()
                .map_err(|e| e.to_string())?,
            ai_price_in_per_mtok: self
                .setting("ai_price_in_per_mtok")?
                .parse::<f64>()
                .map_err(|e| e.to_string())?,
            ai_price_out_per_mtok: self
                .setting("ai_price_out_per_mtok")?
                .parse::<f64>()
                .map_err(|e| e.to_string())?,
        })
    }
    fn setting(&self, key: &str) -> Result<String, String> {
        self.conn
            .query_row("SELECT value FROM settings WHERE key=?1", [key], |row| {
                row.get(0)
            })
            .map_err(|error| error.to_string())
    }
    pub fn save_settings(&mut self, s: &Settings) -> Result<(), String> {
        s.validate()?;
        let tx = self.conn.transaction().map_err(|e| e.to_string())?;
        for (k, v) in [
            ("sync_minutes", s.sync_minutes),
            ("mail_unanswered_days", s.mail_unanswered_days),
        ] {
            tx.execute(
                "INSERT OR REPLACE INTO settings VALUES(?1,?2)",
                params![k, v.to_string()],
            )
            .map_err(|e| e.to_string())?;
        }
        for (key, value) in [
            ("ai_model", s.ai_model.clone()),
            ("ai_monthly_cap_usd", s.ai_monthly_cap_usd.to_string()),
            ("ai_price_in_per_mtok", s.ai_price_in_per_mtok.to_string()),
            ("ai_price_out_per_mtok", s.ai_price_out_per_mtok.to_string()),
        ] {
            tx.execute(
                "INSERT OR REPLACE INTO settings VALUES(?1,?2)",
                params![key, value],
            )
            .map_err(|error| error.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())
    }
    pub fn has_judgment(&self, thread_id: &str, message_id: &str) -> Result<bool, String> {
        self.conn
            .query_row(
                "SELECT 1 FROM ai_judgments WHERE thread_id=?1 AND last_message_id=?2",
                params![thread_id, message_id],
                |_| Ok(()),
            )
            .optional()
            .map(|found| found.is_some())
            .map_err(|error| error.to_string())
    }
    pub fn judgments(&self) -> Result<HashMap<(String, String), Judgment>, String> {
        let mut statement = self
            .conn
            .prepare("SELECT thread_id,last_message_id,needs_action,summary FROM ai_judgments")
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    (row.get(0)?, row.get(1)?),
                    Judgment {
                        needs_action: row.get::<_, i64>(2)? != 0,
                        summary: row.get(3)?,
                    },
                ))
            })
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<_, _>>()
            .map_err(|error| error.to_string())
    }
    pub fn feedback_votes(&self) -> Result<HashMap<(String, String), i8>, String> {
        let mut statement = self
            .conn
            .prepare("SELECT source,item_id,vote FROM feedback")
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| Ok(((row.get(0)?, row.get(1)?), row.get(2)?)))
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<_, _>>()
            .map_err(|error| error.to_string())
    }
    pub fn record_judgment(
        &mut self,
        thread_id: &str,
        message_id: &str,
        result: &ResultWithUsage,
        model: &str,
        cost: f64,
        now: DateTime<Utc>,
    ) -> Result<(), String> {
        let transaction = self.conn.transaction().map_err(|error| error.to_string())?;
        let inserted = transaction
            .execute(
                "INSERT OR IGNORE INTO ai_judgments VALUES(?1,?2,?3,?4,?5)",
                params![
                    thread_id,
                    message_id,
                    i64::from(result.judgment.needs_action),
                    result.judgment.summary,
                    now.timestamp()
                ],
            )
            .map_err(|error| error.to_string())?;
        if inserted == 1 {
            Self::record_usage(
                &transaction,
                model,
                result.input_tokens,
                result.output_tokens,
                cost,
                now,
            )?;
        }
        transaction.commit().map_err(|error| error.to_string())
    }
    fn record_usage(
        conn: &Connection,
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
        cost: f64,
        now: DateTime<Utc>,
    ) -> Result<(), String> {
        conn
            .execute(
                "INSERT INTO ai_usage(model,input_tokens,output_tokens,cost_usd,created_at) VALUES(?1,?2,?3,?4,?5)",
                params![model, input_tokens, output_tokens, cost, now.timestamp()],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }
    pub fn set_feedback(
        &self,
        source: &str,
        item_id: &str,
        vote: i8,
        now: DateTime<Utc>,
    ) -> Result<(), String> {
        if ![-1, 1].contains(&vote) {
            return Err("피드백은 👍 또는 👎만 선택하세요".into());
        }
        self.conn
            .execute(
                "INSERT OR REPLACE INTO feedback VALUES(?1,?2,?3,?4)",
                params![source, item_id, vote, now.timestamp()],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }
    pub fn month_cost(&self, now: DateTime<Local>) -> Result<f64, String> {
        self.conn.query_row("SELECT COALESCE(SUM(cost_usd),0) FROM ai_usage WHERE created_at>=?1 AND created_at<=?2",
            params![ai::month_start_utc(now).timestamp(), now.timestamp()], |row| row.get(0))
            .map_err(|error| error.to_string())
    }
    pub fn stats(&self, now: DateTime<Local>) -> Result<Stats, String> {
        let (cost, input, output): (f64, u64, u64) = self.conn.query_row(
            "SELECT COALESCE(SUM(cost_usd),0),COALESCE(SUM(input_tokens),0),COALESCE(SUM(output_tokens),0) FROM ai_usage WHERE created_at>=?1 AND created_at<=?2",
            params![ai::month_start_utc(now).timestamp(), now.timestamp()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).map_err(|error| error.to_string())?;
        let (up, down): (u64, u64) = self.conn.query_row(
            "SELECT COALESCE(SUM(CASE WHEN vote=1 THEN 1 ELSE 0 END),0),COALESCE(SUM(CASE WHEN vote=-1 THEN 1 ELSE 0 END),0) FROM feedback WHERE created_at>=?1 AND created_at<=?2",
            params![(now - Duration::days(14)).timestamp(), now.timestamp()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(|error| error.to_string())?;
        let total = up + down;
        Ok(Stats {
            month_cost_usd: cost,
            month_input_tokens: input,
            month_output_tokens: output,
            cap_usd: self.settings()?.ai_monthly_cap_usd,
            feedback_14d: FeedbackStats {
                up,
                down,
                ratio: if total == 0 {
                    0.0
                } else {
                    up as f64 / total as f64
                },
            },
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{fixture, ItemKind};
    #[test]
    fn replacement_is_atomic_and_source_scoped() {
        let mut s = Store::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        let a = fixture("a", ItemKind::Task, chrono::Utc::now());
        s.replace_items("test", std::slice::from_ref(&a)).unwrap();
        let mut other = a.clone();
        other.source = "other".into();
        s.replace_items("other", &[other]).unwrap();
        let b = fixture("b", ItemKind::Task, chrono::Utc::now());
        assert!(s.replace_items("test", &[b.clone(), b.clone()]).is_err());
        assert!(s
            .items()
            .unwrap()
            .iter()
            .any(|i| i.id == "a" && i.source == "test"));
        s.replace_items("test", &[b]).unwrap();
        assert_eq!(s.items().unwrap().len(), 2);
        s.replace_items("test", &[]).unwrap();
        assert_eq!(s.items().unwrap()[0].source, "other");
        assert_eq!(s.settings().unwrap().sync_minutes, 15);
    }
    #[test]
    fn notion_mapping_round_trip() {
        let store = Store::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        assert!(store.notion_mapping().unwrap().is_none());
        store
            .save_notion_mapping("{\"data_source_id\":\"ds\"}")
            .unwrap();
        assert_eq!(
            store.notion_mapping().unwrap().as_deref(),
            Some("{\"data_source_id\":\"ds\"}")
        );
        store.clear_notion_mapping().unwrap();
        assert!(store.notion_mapping().unwrap().is_none());
    }
    #[test]
    fn github_login_and_review_days() {
        let store = Store::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        assert_eq!(store.github_review_days().unwrap(), 1);
        assert!(store.github_login().unwrap().is_none());
        store.save_github_login("octocat").unwrap();
        assert_eq!(store.github_login().unwrap().as_deref(), Some("octocat"));
        store.clear_github_login().unwrap();
        assert!(store.github_login().unwrap().is_none());
    }
    #[test]
    fn judgment_is_cached_once_and_month_stats_use_local_boundary() {
        use chrono::TimeZone;
        let mut store = Store::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        let now = Local.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap();
        let result = ResultWithUsage {
            judgment: Judgment {
                needs_action: true,
                summary: Some("답장".into()),
            },
            input_tokens: 100,
            output_tokens: 10,
        };
        let before = now - Duration::seconds(1);
        store
            .record_judgment(
                "old",
                "m1",
                &result,
                "model",
                0.001,
                before.with_timezone(&Utc),
            )
            .unwrap();
        store
            .record_judgment(
                "new",
                "m2",
                &result,
                "model",
                0.002,
                now.with_timezone(&Utc),
            )
            .unwrap();
        store
            .record_judgment(
                "new",
                "m2",
                &result,
                "model",
                0.002,
                now.with_timezone(&Utc),
            )
            .unwrap();
        assert!(store.has_judgment("new", "m2").unwrap());
        assert!(!store.has_judgment("new", "changed").unwrap());
        assert_eq!(
            store.judgments().unwrap()[&("new".into(), "m2".into())]
                .summary
                .as_deref(),
            Some("답장")
        );
        let stats = store.stats(now + Duration::seconds(1)).unwrap();
        assert!((stats.month_cost_usd - 0.002).abs() < 1e-9);
        assert_eq!(stats.month_input_tokens, 100);
        assert_eq!(stats.month_output_tokens, 10);
        assert!((store.month_cost(now + Duration::seconds(1)).unwrap() - 0.002).abs() < 1e-9);
    }
    #[test]
    fn usage_without_judgment_is_counted() {
        let store = Store::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        let now = Utc::now();
        Store::record_usage(&store.conn, "model", 100, 10, 0.002, now).unwrap();
        assert!(!store.has_judgment("thread", "message").unwrap());
        let stats = store.stats(now.with_timezone(&Local)).unwrap();
        assert!((stats.month_cost_usd - 0.002).abs() < 1e-9);
        assert_eq!(stats.month_input_tokens, 100);
        assert_eq!(stats.month_output_tokens, 10);
    }
    #[test]
    fn feedback_counts_and_settings_round_trip() {
        let mut store = Store::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        let now = Local::now();
        let mut settings = store.settings().unwrap();
        assert_eq!(settings.ai_model, "claude-haiku-4-5-20251001");
        assert_eq!(settings.ai_monthly_cap_usd, 5.0);
        settings.ai_monthly_cap_usd = 0.0001;
        settings.ai_model = "test-model".into();
        store.save_settings(&settings).unwrap();
        assert_eq!(store.settings().unwrap().ai_model, "test-model");
        store
            .set_feedback("gmail", "a", 1, now.with_timezone(&Utc))
            .unwrap();
        store
            .set_feedback("github", "b", -1, now.with_timezone(&Utc))
            .unwrap();
        assert_eq!(
            store.feedback_votes().unwrap()[&("github".into(), "b".into())],
            -1
        );
        let stats = store.stats(now).unwrap();
        assert_eq!((stats.feedback_14d.up, stats.feedback_14d.down), (1, 1));
        assert_eq!(stats.feedback_14d.ratio, 0.5);
        assert!(store
            .set_feedback("gmail", "a", 0, now.with_timezone(&Utc))
            .is_err());
    }
}
