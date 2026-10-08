mod commands;
mod connectors;
mod missed;
mod model;
mod oauth;
mod secrets;
mod store;
mod summary;
mod sync;
use std::sync::{Arc, Mutex};
use tauri::Manager;
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let directory = app.path().app_data_dir()?;
            std::fs::create_dir_all(&directory)?;
            let store = store::Store::open(&directory.join("cache.sqlite"))
                .map_err(std::io::Error::other)?;
            let store = Arc::new(Mutex::new(store));
            let auth = Arc::new(oauth::GoogleAuth::default());
            let state = Arc::new(sync::AppState {
                store: store.clone(),
                registry: connectors::registry(auth.clone(), store),
                auth,
                operation: tokio::sync::Mutex::new(()),
            });
            app.manage(state.clone());
            tauri::async_runtime::spawn(sync::run(state, app.handle().clone()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_dashboard,
            commands::get_items,
            commands::sync_now,
            commands::connect_google,
            commands::disconnect_google,
            commands::save_google_client,
            commands::has_google_client,
            commands::save_notion,
            commands::disconnect_notion,
            commands::get_settings,
            commands::save_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running application");
}
