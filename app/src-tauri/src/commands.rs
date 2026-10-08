use crate::{
    ai,
    connectors::github::GitHub,
    connectors::notion::Notion,
    missed::{self, Context, MissedItem, Rules},
    model::{Item, ItemKind, Settings, SourceStatus, Stage},
    secrets,
    store::{Stats, Store},
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
    let mut items = store.items()?;
    let settings = store.settings()?;
    let context = Context {
        judgments: store.judgments()?,
        feedback: store.feedback_votes()?,
    };
    apply_ai_stages(&mut items, &context);
    let missed = missed::missed(
        &items,
        chrono::Utc::now(),
        &Rules {
            mail_unanswered_days: settings.mail_unanswered_days,
            github_review_days: store.github_review_days()?,
        },
        &context,
    );
    Ok(Dashboard {
        summary: summary::summarize(&items, &missed, chrono::Local::now(), period_days),
        missed,
        statuses: store.statuses()?,
    })
}
fn apply_ai_stages(items: &mut [Item], context: &Context) {
    for item in items {
        if item.kind != ItemKind::Email
            || context
                .feedback
                .get(&(item.source.clone(), item.id.clone()))
                == Some(&-1)
        {
            continue;
        }
        let Some((thread, message)) = item.meta["thread_id"]
            .as_str()
            .zip(item.meta["last_message_id"].as_str())
        else {
            continue;
        };
        if context
            .judgments
            .get(&(thread.to_owned(), message.to_owned()))
            .is_some_and(|judgment| judgment.needs_action)
        {
            item.stage = Some(Stage::Requested);
        }
    }
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
pub async fn save_notion(
    state: State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
    token: String,
    database: String,
) -> Result<(), String> {
    let token = token.trim();
    if token.is_empty() {
        return Err("Notion 토큰을 입력하세요".into());
    }
    let _guard = state.operation.lock().await;
    let mapping = Notion::validate_and_map(token, &database).await?;
    let mapping_json = serde_json::to_string(&mapping).map_err(|_| "Notion 매핑 형식 오류")?;
    let previous_token = secrets::get("notion.token")?;
    secrets::set("notion.token", token)?;
    if let Err(error) = state
        .store()
        .and_then(|store| store.save_notion_mapping(&mapping_json))
    {
        match previous_token {
            Some(value) => secrets::set("notion.token", &value)?,
            None => secrets::delete("notion.token")?,
        }
        return Err(error);
    }
    drop(_guard);
    state.sync(&app).await
}
#[tauri::command]
pub async fn disconnect_notion(
    state: State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let _guard = state.operation.lock().await;
    secrets::delete("notion.token")?;
    let mut store = state.store()?;
    store.clear_notion_mapping()?;
    store.replace_items("notion", &[])?;
    store.save_status(&SourceStatus {
        source: "notion".into(),
        ..Default::default()
    })?;
    AppState::notify(&app);
    Ok(())
}
#[tauri::command]
pub async fn save_github(
    state: State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
    token: String,
) -> Result<(), String> {
    let token = token.trim();
    if token.is_empty() {
        return Err("GitHub 토큰을 입력하세요".into());
    }
    let guard = state.operation.lock().await;
    let login = GitHub::validate_login(token).await?;
    let previous_token = secrets::get("github.token")?;
    secrets::set("github.token", token)?;
    if let Err(error) = state
        .store()
        .and_then(|store| store.save_github_login(&login))
    {
        match previous_token {
            Some(value) => secrets::set("github.token", &value)?,
            None => secrets::delete("github.token")?,
        }
        return Err(error);
    }
    drop(guard);
    state.sync(&app).await
}
#[tauri::command]
pub async fn disconnect_github(
    state: State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let _guard = state.operation.lock().await;
    secrets::delete("github.token")?;
    let mut store = state.store()?;
    store.clear_github_login()?;
    store.replace_items("github", &[])?;
    store.save_status(&SourceStatus {
        source: "github".into(),
        ..Default::default()
    })?;
    AppState::notify(&app);
    Ok(())
}
#[tauri::command]
pub fn get_github_login(state: State<'_, Arc<AppState>>) -> Result<Option<String>, String> {
    state.store()?.github_login()
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
#[tauri::command]
pub fn save_anthropic_key(
    state: State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
    key: String,
) -> Result<(), String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("Anthropic API 키를 입력하세요".into());
    }
    let _guard = state
        .operation
        .try_lock()
        .map_err(|_| "동기화 중에는 키를 변경할 수 없습니다")?;
    secrets::set("anthropic.api_key", key)?;
    AppState::notify(&app);
    Ok(())
}
#[tauri::command]
pub fn set_feedback(
    state: State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
    source: String,
    item_id: String,
    vote: i8,
) -> Result<(), String> {
    state
        .store()?
        .set_feedback(&source, &item_id, vote, chrono::Utc::now())?;
    AppState::notify(&app);
    Ok(())
}
#[derive(Serialize)]
pub struct StatsResponse {
    #[serde(flatten)]
    stats: Stats,
    key_last4: Option<String>,
    cap_reached: bool,
}
#[tauri::command]
pub fn get_stats(state: State<'_, Arc<AppState>>) -> Result<StatsResponse, String> {
    let (stats, settings) = {
        let store = state.store()?;
        (store.stats(chrono::Local::now())?, store.settings()?)
    };
    let key_last4 = secrets::get("anthropic.api_key")?.map(|key| {
        key.chars()
            .rev()
            .take(4)
            .collect::<String>()
            .chars()
            .rev()
            .collect()
    });
    let cap_reached = cap_reached(stats.month_cost_usd, &settings);
    Ok(StatsResponse {
        stats,
        key_last4,
        cap_reached,
    })
}
fn cap_reached(spent: f64, settings: &Settings) -> bool {
    let empty_mail = Item {
        id: String::new(),
        source: "gmail".into(),
        kind: ItemKind::Email,
        title: String::new(),
        subtitle: None,
        url: String::new(),
        timestamp: chrono::Utc::now(),
        due: None,
        done: false,
        stage: None,
        meta: serde_json::json!({"snippet":""}),
    };
    let worst = ai::worst_case_cost(
        &ai::request_body(&settings.ai_model, &empty_mail),
        settings.ai_price_in_per_mtok,
        settings.ai_price_out_per_mtok,
    );
    !ai::fits_cap(spent, 0.0, worst, settings.ai_monthly_cap_usd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::Judgment;
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
    #[test]
    fn ai_required_email_gets_requested_stage() {
        let mut email = fixture("thread", ItemKind::Email, Utc::now());
        email.source = "gmail".into();
        email.meta["thread_id"] = "thread".into();
        email.meta["last_message_id"] = "message".into();
        let mut context = Context::default();
        context.judgments.insert(
            ("thread".into(), "message".into()),
            Judgment {
                needs_action: true,
                summary: Some("답장".into()),
            },
        );
        apply_ai_stages(std::slice::from_mut(&mut email), &context);
        assert_eq!(email.stage, Some(Stage::Requested));
        context
            .feedback
            .insert(("gmail".into(), "thread".into()), -1);
        email.stage = None;
        apply_ai_stages(std::slice::from_mut(&mut email), &context);
        assert!(email.stage.is_none());
    }
    #[test]
    fn cap_reached_when_minimal_request_will_not_fit() {
        let mut settings = Settings::default();
        assert!(!cap_reached(0.0, &settings));
        settings.ai_monthly_cap_usd = 0.0;
        assert!(cap_reached(0.0, &settings));
        settings.ai_monthly_cap_usd = 5.0;
        assert!(cap_reached(5.0, &settings));
    }
}
