use crate::{
    missed::{self, MissedItem, Rules},
    model::{Item, ItemKind, Settings, SourceStatus},
    secrets,
    store::Store,
    summary::{self, Summary},
    sync::AppState,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
#[derive(Serialize)]
pub struct Dashboard {
    summary: Summary,
    missed: Vec<MissedItem>,
    statuses: Vec<SourceStatus>,
}
#[derive(Default, Deserialize)]
pub struct ItemFilter {
    pub query: Option<String>,
}
#[tauri::command]
pub fn get_dashboard(
    state: State<'_, Arc<AppState>>,
    period_days: u32,
) -> Result<Dashboard, String> {
    if ![1, 7, 30].contains(&period_days) {
        return Err("기간은 1, 7, 30일 중 선택하세요".into());
    }
    let store = state.store()?;
    let items = store.items()?;
    let settings = store.settings()?;
    let missed = missed::missed(
        &items,
        chrono::Utc::now(),
        &Rules {
            mail_unanswered_days: settings.mail_unanswered_days,
        },
    );
    Ok(Dashboard {
        summary: summary::summarize(&items, &missed, chrono::Local::now(), period_days),
        missed,
        statuses: store.statuses()?,
    })
}
#[tauri::command]
pub fn get_items(
    state: State<'_, Arc<AppState>>,
    filter: Option<ItemFilter>,
) -> Result<Vec<Item>, String> {
    let query = filter
        .unwrap_or_default()
        .query
        .unwrap_or_default()
        .to_lowercase();
    let mut items: Vec<_> = state
        .store()?
        .items()?
        .into_iter()
        .filter(|i| {
            !i.done
                && matches!(
                    i.kind,
                    ItemKind::Task | ItemKind::Issue | ItemKind::PullRequest
                )
                && i.title.to_lowercase().contains(&query)
        })
        .collect();
    items.sort_by_key(|i| std::cmp::Reverse(i.timestamp));
    Ok(items)
}
#[tauri::command]
pub async fn sync_now(
    state: State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    state.sync(&app).await
}
#[tauri::command]
pub async fn connect_google(
    state: State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    {
        let _guard = state
            .operation
            .try_lock()
            .map_err(|_| "다른 연결 또는 동기화 작업이 진행 중입니다")?;
        state.auth.connect(&app).await?;
    }
    state.sync(&app).await
}
#[tauri::command]
pub async fn disconnect_google(
    state: State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let _guard = state.operation.lock().await;
    state.auth.disconnect().await?;
    let mut store = state.store()?;
    clear_google_cache(&mut store)?;
    AppState::notify(&app);
    Ok(())
}
fn clear_google_cache(store: &mut Store) -> Result<(), String> {
    for source in ["gmail", "calendar"] {
        store.replace_items(source, &[])?;
        store.save_status(&SourceStatus {
            source: source.into(),
            ..Default::default()
        })?;
    }
    Ok(())
}
#[tauri::command]
pub async fn save_google_client(
    state: State<'_, Arc<AppState>>,
    client_id: String,
    client_secret: String,
) -> Result<(), String> {
    let _guard = state
        .operation
        .try_lock()
        .map_err(|_| "연결 작업이 진행 중입니다")?;
    if state.auth.is_connected() {
        return Err("연결 해제 후 클라이언트 정보를 변경하세요".into());
    }
    if client_id.trim().is_empty() || client_secret.trim().is_empty() {
        return Err("클라이언트 ID와 secret을 입력하세요".into());
    }
    secrets::set("google.client_id", client_id.trim())?;
    secrets::set("google.client_secret", client_secret.trim())
}
#[tauri::command]
pub fn has_google_client() -> Result<bool, String> {
    Ok(secrets::get("google.client_id")?.is_some()
        && secrets::get("google.client_secret")?.is_some())
}
#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> Result<Settings, String> {
    state.store()?.settings()
}
#[tauri::command]
pub fn save_settings(
    state: State<'_, Arc<AppState>>,
    settings: Settings,
    app: tauri::AppHandle,
) -> Result<(), String> {
    state.store()?.save_settings(&settings)?;
    AppState::notify(&app);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::fixture;
    use chrono::Utc;
    use std::path::Path;

    #[test]
    fn google_disconnect_preserves_other_sources() {
        let mut store = Store::open(Path::new(":memory:")).unwrap();
        for source in ["gmail", "calendar", "notion", "github"] {
            let mut item = fixture(source, ItemKind::Task, Utc::now());
            item.source = source.into();
            store.replace_items(source, &[item]).unwrap();
            store
                .save_status(&SourceStatus {
                    source: source.into(),
                    connected: true,
                    last_synced: Some(Utc::now()),
                    error: Some("saved status".into()),
                    ..Default::default()
                })
                .unwrap();
        }

        clear_google_cache(&mut store).unwrap();

        let items = store.items().unwrap();
        let statuses = store.statuses().unwrap();
        for source in ["gmail", "calendar"] {
            assert!(!items.iter().any(|item| item.source == source));
            let status = statuses
                .iter()
                .find(|status| status.source == source)
                .unwrap();
            assert!(!status.connected);
            assert!(status.last_synced.is_none());
            assert!(status.error.is_none());
        }
        for source in ["notion", "github"] {
            assert!(items.iter().any(|item| item.source == source));
            let status = statuses
                .iter()
                .find(|status| status.source == source)
                .unwrap();
            assert!(status.connected);
            assert!(status.last_synced.is_some());
            assert_eq!(status.error.as_deref(), Some("saved status"));
        }
    }
}
