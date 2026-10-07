use crate::model::{Item, Settings, SourceStatus};
use rusqlite::{params, Connection};
use std::path::Path;

pub struct Store {
    conn: Connection,
}
impl Store {
    pub fn open(path: &Path) -> Result<Self, String> {
        Self::from_connection(Connection::open(path).map_err(|e| e.to_string())?)
    }
    fn from_connection(conn: Connection) -> Result<Self, String> {
        conn.execute_batch("CREATE TABLE IF NOT EXISTS items(source TEXT NOT NULL,id TEXT NOT NULL,json TEXT NOT NULL,PRIMARY KEY(source,id)); CREATE TABLE IF NOT EXISTS source_status(source TEXT PRIMARY KEY,json TEXT NOT NULL); CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY,value TEXT NOT NULL); INSERT OR IGNORE INTO settings VALUES('sync_minutes','15'),('mail_unanswered_days','2');").map_err(|e|e.to_string())?;
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
        })
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
        tx.commit().map_err(|e| e.to_string())
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
}
