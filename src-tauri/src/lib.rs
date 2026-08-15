mod archive;
mod config;
mod scanner;

use config::AppConfig;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

struct ConfigState(Mutex<AppConfig>);

/// 窗口显隐切换（托盘点击 / 全局热键触发）
fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

#[tauri::command]
fn get_config(state: tauri::State<ConfigState>) -> Result<AppConfig, String> {
    Ok(state.0.lock().unwrap().clone())
}

#[tauri::command]
fn save_config(
    app: AppHandle,
    state: tauri::State<ConfigState>,
    config: AppConfig,
) -> Result<(), String> {
    // 热键变化时重注册
    {
        let mut cfg = state.0.lock().unwrap();
        let hotkey_changed = cfg.hotkey != config.hotkey;
        *cfg = config.clone();
        if hotkey_changed {
            register_hotkey(&app, &config.hotkey)?;
        }
    }
    config.save()
}

#[tauri::command]
fn scan_folders(state: tauri::State<ConfigState>) -> Vec<scanner::FolderEntry> {
    let cfg = state.0.lock().unwrap();
    scanner::scan_roots(&cfg.root_dirs, &cfg.favorites, cfg.scan_depth)
}

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    archive::open_in_explorer(&path)
}

#[tauri::command]
fn archive_files(
    files: Vec<String>,
    dest: String,
) -> Result<Vec<archive::ArchiveResult>, String> {
    archive::archive_files(files, &dest)
}

#[tauri::command]
fn pick_folders() -> Vec<String> {
    rfd::FileDialog::new()
        .set_title("选择文件夹")
        .pick_folders()
        .into_iter()
        .flatten()
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

#[tauri::command]
fn pick_files() -> Vec<String> {
    rfd::FileDialog::new()
        .set_title("选择要归档的文件")
        .pick_files()
        .into_iter()
        .flatten()
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

#[tauri::command]
fn hide_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

fn register_hotkey(app: &AppHandle, hotkey: &str) -> Result<(), String> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
    let _ = app.global_shortcut().unregister_all();
    let shortcut: Shortcut = hotkey.parse().map_err(|e| format!("热键格式错误: {e}"))?;
    app.global_shortcut()
        .on_shortcut(shortcut, move |app, _sc, event| {
            if event.state() == ShortcutState::Pressed {
                toggle_window(app);
            }
        })
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = AppConfig::load();

    tauri::Builder::default()
        .manage(ConfigState(Mutex::new(config)))
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_single_instance::init(|app, _args, _cwd| {
                show_window(app);
            }),
        )
        .plugin(
            tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                Some(vec![]),
            ),
        )
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(|app| {
            // 开机自启
            let cfg = app.state::<ConfigState>();
            let cfg = cfg.0.lock().unwrap().clone();
            #[cfg(not(target_os = "linux"))]
            {
                use tauri_plugin_autostart::ManagerExt;
                if cfg.autostart {
                    let _ = app.autolaunch().enable();
                }
            }

            // 全局热键
            register_hotkey(app.handle(), &cfg.hotkey)?;

            // 系统托盘
            let toggle_item = MenuItem::with_id(app, "toggle", "显示/隐藏面板", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "toggle" => toggle_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        toggle_window(tray.app_handle());
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            scan_folders,
            open_folder,
            archive_files,
            pick_folders,
            pick_files,
            hide_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
