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
    App, AppHandle, Emitter, Manager,
    menu::MenuBuilder,
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    utils::platform::{Target, target_triple},
};
use tokio::task::JoinSet;
use unit_prefix::NumberPrefix;
use uuid::Uuid;

const TRAY_ID: &str = "tray";
const WINDOW_ID: &str = "main";
const SETTINGS_FILE: &str = "settings.json";
static SETTINGS: OnceLock<RwLock<HashSet<Setting>>> = OnceLock::new();

#[derive(Clone, Serialize, Deserialize)]
pub struct Fetched {
    stars: u32,
    setting: Setting,
}

#[tauri::command]
fn uuid() -> String {
    Uuid::new_v4().to_string()
}

#[tauri::command]
async fn create(app: AppHandle, setting: Setting) -> Result<u32> {
    let stars = fetch(setting.repo()).await.map(|(_, stars)| stars)?;

    SETTINGS.get().unwrap().write().unwrap().insert(setting);
    store(app).await?;

    Ok(stars)
}

#[tauri::command]
async fn update(app: AppHandle, setting: Setting) -> Result<()> {
    let mut settings: Vec<Setting> = Vec::new();
    if *setting.favourite() {
        let _ = set_toolbar(&app, setting.repo()).await;
        let mut copied: Vec<Setting> = SETTINGS
            .get()
            .unwrap()
            .read()
            .unwrap()
            .iter()
            .cloned()
            .collect();
        copied.iter_mut().for_each(|f| {
            f.set_favourite(false);
        });
        settings.append(&mut copied);
    }
    {
        let mut data = SETTINGS.get().unwrap().write().unwrap();
        settings.push(setting);
        settings.into_iter().for_each(|f| {
            data.replace(f);
        });
    }
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
async fn read(app: AppHandle) -> Vec<Result<Fetched>> {
    if SETTINGS.get().is_none() {
        let data = load(&settings_file_path(&app)).await.unwrap_or_default();
        let _ = SETTINGS.set(RwLock::new(data));
    }
    let data = SETTINGS.get().unwrap().read().unwrap().clone();
    let mut set = JoinSet::new();
    data.into_iter().for_each(|setting| {
        set.spawn(async move {
            let stars = setting.repo().fetch().await?;
            Ok(Fetched { stars, setting })
        });
    });
    let data = set.join_all().await;
    if let Some(Ok(favourite)) = data
        .iter()
        .find(|f| f.as_ref().is_ok_and(|f| *f.setting.favourite()))
        .as_ref()
    {
        let _ = set_toolbar(&app, favourite.setting.repo()).await;
    }
    data
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if Target::from_triple(&target_triple()?) == Target::MacOS {
                build_tray(app)?;
                hide_window(app.app_handle());
            }

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![uuid, read, create, update, delete])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn fetch(which: &Repo) -> Result<(String, u32)> {
    let stars = which.fetch().await?;
    let project = which.to_string();
    Ok((project, stars))
}

async fn set_toolbar(app: &AppHandle, repo: &Repo) -> Result<()> {
    if let Ok(Target::MacOS) = target_triple().map(|target| Target::from_triple(&target)) {
        let project = repo.name();
        let (_, stars) = fetch(repo).await?;

        let stars = match NumberPrefix::decimal(stars as f64) {
            NumberPrefix::Prefixed(prefix, n) => {
                format!("{n:.0}{prefix}")
            }
            _ => format!("{stars}"),
        };

        let txt = format!("⭐️ {project} {stars}");
        let _ = app.tray_by_id(TRAY_ID).unwrap().set_title(Some(txt));
    }

    Ok(())
}

fn build_tray(app: &mut App) -> Result<(), tauri::Error> {
    let menu = MenuBuilder::new(app)
        .text("open", "Open")
        .text("refresh", "Refresh")
        .text("exit", "Exit")
        .build()?;
    TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .title(format!(
            "⭐️ {}",
            app.config().product_name.as_ref().unwrap()
        ))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button, .. } = event {
                if button != MouseButton::Left {
                    return;
                }
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window(WINDOW_ID) {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .on_menu_event(|app_handle, event| match event.id().0.as_str() {
            "open" => open_window(app_handle),
            "exit" => quit_app(app_handle),
            "refresh" => {
                let _ = app_handle.emit("refresh", ());
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
}

fn quit_app(app: &AppHandle) {
    app.exit(0);
}

fn open_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WINDOW_ID) {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn hide_window(app: &AppHandle) {
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
