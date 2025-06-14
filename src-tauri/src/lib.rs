mod backend;

use backend::{
    Repo, Result,
    settings::{Setting, load},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{OnceLock, RwLock},
};
use tauri::{
    App, AppHandle, Manager,
    menu::Menu,
    tray::{TrayIconBuilder, TrayIconEvent},
};
use tokio::task::JoinSet;
use unit_prefix::NumberPrefix;
use uuid::Uuid;

const TRAY_ID: &str = "tray";
const WINDOW_ID: &str = "main";
const SETTINGS_FILE: &str = "settings.json";
static SETTINGS: OnceLock<RwLock<HashSet<Setting>>> = OnceLock::new();

#[derive(Serialize, Deserialize)]
pub struct Fetched {
    stars: u32,
    setting: Setting,
}
#[tauri::command]
fn uuid() -> String {
    Uuid::new_v4().to_string()
}
#[tauri::command]
async fn fetch(which: Repo) -> Result<(String, u32)> {
    let stars = which.fetch().await?;
    let project = which.to_string();
    Ok((project, stars))
}

#[tauri::command]
async fn set_toolbar(app: AppHandle, repo: Repo) -> Result<()> {
    let project = repo.to_string();
    let (_, stars) = fetch(repo).await?;

    let stars = match NumberPrefix::decimal(stars as f64) {
        NumberPrefix::Prefixed(prefix, n) => {
            format!("{:.0}{}", n, prefix)
        }
        _ => format!("{stars}"),
    };

    let txt = format!("⭐️ {project} {stars}");
    let _ = app.tray_by_id(TRAY_ID).unwrap().set_title(Some(txt));
    Ok(())
}

#[tauri::command]
async fn create(app: AppHandle, setting: Setting) -> Result<u32> {
    let repo = setting.repo().clone();
    SETTINGS.get().unwrap().write().unwrap().insert(setting);
    store(app).await?;
    fetch(repo).await.map(|(_, stars)| stars)
}

#[tauri::command]
async fn update(app: AppHandle, setting: Setting) -> Result<()> {
    SETTINGS.get().unwrap().write().unwrap().replace(setting);
    store(app).await?;
    Ok(())
}

#[tauri::command]
async fn delete(app: AppHandle, setting: Setting) -> Result<()> {
    SETTINGS.get().unwrap().write().unwrap().remove(&setting);
    store(app).await?;
    Ok(())
}

#[tauri::command]
async fn read(app: AppHandle) -> Vec<Result<Fetched, String>> {
    if SETTINGS.get().is_none() {
        let data = load(&settings_file_path(&app)).await.unwrap_or_default();
        let _ = SETTINGS.set(RwLock::new(data));
    }
    let data = SETTINGS.get().unwrap().read().unwrap().clone();
    let mut set = JoinSet::new();
    data.into_iter().for_each(|setting| {
        set.spawn(async move {
            let stars = setting
                .repo()
                .fetch()
                .await
                .map_err(|err| err.to_string())?;
            Ok(Fetched { stars, setting })
        });
    });
    set.join_all().await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            build_tray(app)?;
            hide_window(app);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            set_toolbar,
            uuid,
            read,
            fetch,
            create,
            update,
            delete
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn build_tray(app: &mut App) -> Result<(), tauri::Error> {
    let menu = Menu::new(app)?;
    TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .title(format!(
            "⭐️ {}",
            app.config().product_name.as_ref().unwrap()
        ))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { .. } = event {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window(WINDOW_ID) {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;
    Ok(())
}

fn hide_window(app: &mut App) {
    if let Some(window) = app.get_webview_window(WINDOW_ID) {
        let _ = window.hide();
    }
}

fn settings_file_path(app: &AppHandle) -> PathBuf {
    let mut path = app.path().app_local_data_dir().unwrap();
    path.push(SETTINGS_FILE);
    path
}

async fn store(app: AppHandle) -> Result<()> {
    let settings = SETTINGS.get().unwrap().read().unwrap().clone();
    backend::settings::store(&settings, &settings_file_path(&app)).await
}
