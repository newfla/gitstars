use std::sync::{OnceLock, RwLock};

use fetcher::{github::GitHubFetcher, gitlab::GitLabFetcher};
use settings::{
    GitType, Setting, settings_from_path, store_settings_to_path,
};
use tauri::{
    App, AppHandle, Manager,
    menu::Menu,
    tray::{TrayIconBuilder, TrayIconEvent},
};

use async_trait::async_trait;
use downcast_rs::{Downcast, impl_downcast};
use gitlab::RestError;
use thiserror::Error;
use tokio::io;

mod fetcher;
mod settings;

const TRAY_ID: &str = "tray";
const WINDOW_ID: &str = "main";
static SETTINGS: OnceLock<RwLock<Vec<Setting>>> = OnceLock::new();

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    GitHub(#[from] octocrab::Error),
    #[error(transparent)]
    GitLab(#[from] gitlab::GitlabError),
    #[error(transparent)]
    GitLabApi(#[from] gitlab::api::ApiError<RestError>),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[async_trait]
pub trait Fetcher: Downcast {
    async fn stars(&self) -> Result<u32>;
    fn project(&self) -> String;
}

impl_downcast!(Fetcher);

#[tauri::command]
async fn fetch(which: Setting) -> Result<(String, u32), ()> {
    let fetcher: Box<dyn Fetcher + Send> = match which.git_type() {
        GitType::GitHub => Box::new(
            GitHubFetcher::builder()
                .owner(which.owner())
                .repo(which.repo())
                .build(),
        ),
        GitType::GitLab => Box::new(
            GitLabFetcher::builder()
                .owner(which.owner())
                .repo(which.repo())
                .build(),
        ),
    };
    Ok((fetcher.project(), fetcher.stars().await.unwrap_or(0)))
}

#[tauri::command]
async fn set_current(app: AppHandle, which: Setting) -> Result<String, ()> {
    let (project, stars) = fetch(which).await?;
    let txt = format!("⭐️ {project} - {stars}");
    let _ = app
        .tray_by_id(TRAY_ID)
        .unwrap()
        .set_title(Some(txt.clone()));
    Ok(format!("Hello, {}! You've been greeted from Rust!", txt))
}

#[tauri::command]
async fn add(setting: Setting) -> Result<u32, ()> {
    SETTINGS.get().unwrap().write().unwrap().push(setting.clone());
    fetch(setting).await.map(|(_, stars)|stars)
}

#[tauri::command]
async fn store(app: AppHandle) -> Result<(), ()> {
    let settings = SETTINGS.get().unwrap().read().unwrap().clone();
    let res = store_settings_to_path(&settings, &app.path().local_data_dir().unwrap())
        .await
        .map_err(|_| ());
    res
}

#[tauri::command]
async fn load(app: AppHandle) -> Vec<Setting> {
    if SETTINGS.get().is_none() {
        let data = settings_from_path(&app.path().local_data_dir().unwrap())
            .await
            .unwrap_or_default();
        let _ = SETTINGS.set(RwLock::new(data));
    }
    SETTINGS.get().unwrap().read().unwrap().to_vec()
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
        .invoke_handler(tauri::generate_handler![set_current, load, store, fetch, add])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn build_tray(app: &mut App) -> Result<(), tauri::Error> {
    let menu = Menu::new(app)?;
    TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .title(app.config().product_name.as_ref().unwrap())
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
