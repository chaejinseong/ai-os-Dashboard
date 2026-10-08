use crate::{
    ai,
    connectors::Registry,
    model::{Item, ItemKind, SourceStatus},
    oauth::GoogleAuth,
    secrets,
    store::Store,
};
use chrono::{Duration, Utc};
use std::sync::{Arc, Mutex};
use tauri::Emitter;
pub struct AppState {
    pub store: Arc<Mutex<Store>>,
    pub registry: Registry,
    pub auth: Arc<GoogleAuth>,
    pub operation: tokio::sync::Mutex<()>,
}
impl AppState {
    pub fn store(&self) -> Result<std::sync::MutexGuard<'_, Store>, String> {
        self.store.lock().map_err(|_| "캐시 잠금 실패".into())
    }
    pub fn notify(app: &tauri::AppHandle) {
        let _ = app.emit("dashboard-updated", ());
    }
    pub async fn sync(&self, app: &tauri::AppHandle) -> Result<(), String> {
        let Ok(_guard) = self.operation.try_lock() else {
            return Ok(());
        };
        for connector in &self.registry {
            let connected = connector.is_connected().await;
            let mut status = self
                .store()?
                .statuses()?
                .into_iter()
                .find(|s| s.source == connector.id())
                .unwrap_or_else(|| SourceStatus {
                    source: connector.id().into(),
                    ..Default::default()
                });
            status.connected = connected;
            status.syncing = connected;
            if !connected {
                status.error = None;
            }
            self.store()?.save_status(&status)?;
            Self::notify(app);
            if !connected {
                continue;
            }
            match connector.fetch().await {
                Ok(items) => {
                    let replacement = {
                        let mut store = self.store()?;
                        store.replace_items(connector.id(), &items)
                    };
                    match replacement {
                        Ok(()) => {
                            status.last_synced = Some(chrono::Utc::now());
                            status.error = None;
                            if connector.id() == "gmail" {
                                if let Err(error) = self.judge_mail_candidates(&items, app).await {
                                    eprintln!("AI 판정 단계 오류: {error}");
                                }
                            }
                        }
                        Err(e) => status.error = Some(e),
                    }
                }
                Err(e) => status.error = Some(format!("{}: {e}", connector.label())),
            }
            status.syncing = false;
            self.store()?.save_status(&status)?;
            Self::notify(app);
        }
        Ok(())
    }
    async fn judge_mail_candidates(
        &self,
        items: &[Item],
        app: &tauri::AppHandle,
    ) -> Result<(), String> {
        let settings = self.store()?.settings()?;
        let now = Utc::now();
        let mut pending = Vec::new();
        {
            let store = self.store()?;
            for item in items
                .iter()
                .filter(|item| mail_candidate(item, now, settings.mail_unanswered_days))
            {
                let Some((thread, message)) = item.meta["thread_id"]
                    .as_str()
                    .zip(item.meta["last_message_id"].as_str())
                else {
                    continue;
                };
                if !store.has_judgment(thread, message)? {
                    pending.push(item.clone());
                }
            }
        }
        if pending.is_empty()
            || !ai::under_cap(
                self.store()?
                    .month_cost(now.with_timezone(&chrono::Local))?,
                settings.ai_monthly_cap_usd,
            )
        {
            return Ok(());
        }
        let Some(key) = secrets::get("anthropic.api_key")? else {
            return Ok(());
        };
        let client = reqwest::Client::new();
        let mut remaining = pending.into_iter();
        let mut tasks = tokio::task::JoinSet::new();
        loop {
            while tasks.len() < 2
                && ai::under_cap(
                    self.store()?.month_cost(chrono::Local::now())?,
                    settings.ai_monthly_cap_usd,
                )
            {
                let Some(item) = remaining.next() else {
                    break;
                };
                let (key, model, client) = (key.clone(), settings.ai_model.clone(), client.clone());
                tasks.spawn(async move {
                    let result = ai::judge(&client, &key, &model, &item).await;
                    (item, result)
                });
            }
            let Some(completed) = tasks.join_next().await else {
                break;
            };
            match completed {
                Ok((item, Ok(result))) => {
                    let Some((thread, message)) = item.meta["thread_id"]
                        .as_str()
                        .zip(item.meta["last_message_id"].as_str())
                    else {
                        continue;
                    };
                    let cost = ai::cost_usd(
                        result.input_tokens,
                        result.output_tokens,
                        settings.ai_price_in_per_mtok,
                        settings.ai_price_out_per_mtok,
                    );
                    self.store()?.record_judgment(
                        thread,
                        message,
                        &result,
                        &settings.ai_model,
                        cost,
                        Utc::now(),
                    )?;
                    Self::notify(app);
                }
                Ok((_, Err(error))) => eprintln!("AI 판정 요청 오류: {error}"),
                Err(_) => eprintln!("AI 판정 작업 중단"),
            }
        }
        Ok(())
    }
}
fn mail_candidate(item: &Item, now: chrono::DateTime<Utc>, days: u32) -> bool {
    item.kind == ItemKind::Email
        && item.meta["last_from_me"].as_bool() == Some(false)
        && now - item.timestamp >= Duration::days(days.into())
}
pub async fn run(state: Arc<AppState>, app: tauri::AppHandle) {
    let _ = state.sync(&app).await;
    let mut last = std::time::Instant::now();
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let minutes = state
            .store()
            .and_then(|s| s.settings())
            .map(|s| s.sync_minutes)
            .unwrap_or(15);
        if last.elapsed() >= std::time::Duration::from_secs(u64::from(minutes) * 60) {
            let _ = state.sync(&app).await;
            last = std::time::Instant::now();
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::fixture;
    #[test]
    fn mail_candidate_uses_rule_boundary_and_sender() {
        let now = Utc::now();
        let mut edge = fixture("edge", ItemKind::Email, now - Duration::days(2));
        assert!(mail_candidate(&edge, now, 2));
        edge.timestamp += Duration::seconds(1);
        assert!(!mail_candidate(&edge, now, 2));
        edge.timestamp -= Duration::seconds(1);
        edge.meta["last_from_me"] = true.into();
        assert!(!mail_candidate(&edge, now, 2));
    }
}
