use crate::{connectors::Registry, model::SourceStatus, oauth::GoogleAuth, store::Store};
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
                Ok(items) => match self.store()?.replace_items(connector.id(), &items) {
                    Ok(()) => {
                        status.last_synced = Some(chrono::Utc::now());
                        status.error = None;
                    }
                    Err(e) => status.error = Some(e),
                },
                Err(e) => status.error = Some(format!("{}: {e}", connector.label())),
            }
            status.syncing = false;
            self.store()?.save_status(&status)?;
            Self::notify(app);
        }
        Ok(())
    }
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
