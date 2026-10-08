use crate::{
    ai,
    connectors::Registry,
    model::{Item, ItemKind, Settings, SourceStatus},
    oauth::GoogleAuth,
    secrets,
    store::Store,
};
use chrono::{Duration, Utc};
use std::{
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
};
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
        if pending.is_empty() {
            return Ok(());
        }
        let Some(key) = secrets::get("anthropic.api_key")? else {
            return Ok(());
        };
        let spent = self
            .store()?
            .month_cost(now.with_timezone(&chrono::Local))?;
        let client = reqwest::Client::new();
        let model = settings.ai_model.clone();
        run_capped(
            pending,
            spent,
            &settings,
            move |item| {
                let (key, model, client) = (key.clone(), model.clone(), client.clone());
                async move {
                    let result = ai::judge(&client, &key, &model, &item).await;
                    (item, result)
                }
            },
            |item, result, cost| {
                let Some((thread, message)) = item.meta["thread_id"]
                    .as_str()
                    .zip(item.meta["last_message_id"].as_str())
                else {
                    return Ok(());
                };
                self.store()?.record_judgment(
                    thread,
                    message,
                    result,
                    &settings.ai_model,
                    cost,
                    Utc::now(),
                )?;
                Self::notify(app);
                Ok(())
            },
        )
        .await?;
        Ok(())
    }
}
fn mail_candidate(item: &Item, now: chrono::DateTime<Utc>, days: u32) -> bool {
    item.kind == ItemKind::Email
        && item.meta["last_from_me"].as_bool() == Some(false)
        && now - item.timestamp >= Duration::days(days.into())
}
async fn run_capped<J, F, D>(
    pending: Vec<Item>,
    mut spent: f64,
    settings: &Settings,
    judge: J,
    mut on_done: D,
) -> Result<f64, String>
where
    J: Fn(Item) -> F,
    F: Future<Output = (Item, Result<ai::ResultWithUsage, ai::JudgeError>)> + Send + 'static,
    D: FnMut(&Item, &ai::ResultWithUsage, f64) -> Result<(), String>,
{
    let mut remaining = pending.into_iter().peekable();
    let mut reserved = 0.0;
    let mut tasks = tokio::task::JoinSet::new();
    let mut reservations: HashMap<tokio::task::Id, f64> = HashMap::new();
    loop {
        while tasks.len() < 2 {
            let Some(item) = remaining.peek() else {
                break;
            };
            let worst = ai::worst_case_cost(
                &ai::request_body(&settings.ai_model, item),
                settings.ai_price_in_per_mtok,
                settings.ai_price_out_per_mtok,
            );
            if !ai::fits_cap(spent, reserved, worst, settings.ai_monthly_cap_usd) {
                break;
            }
            let item = remaining.next().expect("peeked candidate");
            reserved += worst;
            debug_assert!(spent + reserved <= settings.ai_monthly_cap_usd);
            let task = judge(item);
            let handle = tasks.spawn(async move {
                let (item, result) = task.await;
                (item, worst, result)
            });
            reservations.insert(handle.id(), worst);
        }
        let Some(completed) = tasks.join_next_with_id().await else {
            break;
        };
        match completed {
            Ok((id, (item, worst, result))) => {
                reservations.remove(&id);
                reserved = (reserved - worst).max(0.0);
                match result {
                    Ok(result) => {
                        let cost = ai::cost_usd(
                            result.input_tokens,
                            result.output_tokens,
                            settings.ai_price_in_per_mtok,
                            settings.ai_price_out_per_mtok,
                        );
                        spent += cost;
                        on_done(&item, &result, cost)?;
                    }
                    Err(ai::JudgeError::Status(code)) => eprintln!("AI 요청 실패 ({code})"),
                    Err(ai::JudgeError::Network) => {
                        spent += worst;
                        eprintln!("AI 네트워크 또는 응답 오류");
                    }
                }
            }
            Err(error) => {
                let worst = reservations.remove(&error.id()).unwrap_or(0.0);
                reserved = (reserved - worst).max(0.0);
                spent += worst;
                eprintln!("AI 판정 작업 중단");
            }
        }
        debug_assert!(spent + reserved <= settings.ai_monthly_cap_usd);
    }
    Ok(spent)
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
    use std::sync::atomic::{AtomicUsize, Ordering};
    fn max_usage(settings: &Settings, item: &Item) -> ai::ResultWithUsage {
        let body = ai::request_body(&settings.ai_model, item);
        ai::ResultWithUsage {
            judgment: ai::Judgment {
                needs_action: true,
                summary: None,
            },
            input_tokens: body["system"].as_str().unwrap().len() as u64
                + body["messages"][0]["content"].as_str().unwrap().len() as u64
                + ai::INPUT_OVERHEAD_TOKENS,
            output_tokens: 150,
        }
    }
    fn candidates() -> Vec<Item> {
        vec![
            fixture("one", ItemKind::Email, Utc::now()),
            fixture("two", ItemKind::Email, Utc::now()),
        ]
    }
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
    #[tokio::test]
    async fn cap_holds_with_two_concurrent_candidates() {
        let mut settings = Settings::default();
        let items = candidates();
        let worst = ai::worst_case_cost(
            &ai::request_body(&settings.ai_model, &items[0]),
            settings.ai_price_in_per_mtok,
            settings.ai_price_out_per_mtok,
        );
        settings.ai_monthly_cap_usd = worst * 1.5;
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let model = settings.clone();
        let mut recorded = 0.0;
        let spent = run_capped(
            items,
            0.0,
            &settings,
            move |item| {
                let counter = counter.clone();
                let result = max_usage(&model, &item);
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    (item, Ok(result))
                }
            },
            |_, _, cost| {
                recorded += cost;
                Ok(())
            },
        )
        .await
        .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!((spent - worst).abs() < 1e-9);
        assert!(recorded <= settings.ai_monthly_cap_usd);
    }
    #[tokio::test]
    async fn released_reservation_admits_next_candidate() {
        let mut settings = Settings::default();
        let items = candidates();
        let worst = ai::worst_case_cost(
            &ai::request_body(&settings.ai_model, &items[0]),
            settings.ai_price_in_per_mtok,
            settings.ai_price_out_per_mtok,
        );
        settings.ai_monthly_cap_usd = worst * 1.5;
        let calls = Arc::new(AtomicUsize::new(0));
        let active = Arc::new(AtomicUsize::new(0));
        let peak = Arc::new(AtomicUsize::new(0));
        let (counter, active_copy, peak_copy) = (calls.clone(), active.clone(), peak.clone());
        let model = settings.clone();
        let mut recorded = 0.0;
        let spent = run_capped(
            items,
            0.0,
            &settings,
            move |item| {
                let (counter, active, peak) =
                    (counter.clone(), active_copy.clone(), peak_copy.clone());
                let mut result = max_usage(&model, &item);
                result.input_tokens /= 5;
                result.output_tokens /= 5;
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(current, Ordering::SeqCst);
                    tokio::task::yield_now().await;
                    active.fetch_sub(1, Ordering::SeqCst);
                    (item, Ok(result))
                }
            },
            |_, _, cost| {
                recorded += cost;
                Ok(())
            },
        )
        .await
        .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        assert!(peak.load(Ordering::SeqCst) <= 2);
        assert!(spent <= settings.ai_monthly_cap_usd);
        assert!((spent - recorded).abs() < 1e-9);
    }
    #[tokio::test]
    async fn network_error_keeps_reservation() {
        let mut settings = Settings::default();
        let items = candidates();
        let worst = ai::worst_case_cost(
            &ai::request_body(&settings.ai_model, &items[0]),
            settings.ai_price_in_per_mtok,
            settings.ai_price_out_per_mtok,
        );
        settings.ai_monthly_cap_usd = worst * 1.5;
        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let spent = run_capped(
            items,
            0.0,
            &settings,
            move |item| {
                let counter = counter.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    (item, Err(ai::JudgeError::Network))
                }
            },
            |_, _, _| panic!("network error must not record usage"),
        )
        .await
        .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!((spent - worst).abs() < 1e-9);
    }
}
