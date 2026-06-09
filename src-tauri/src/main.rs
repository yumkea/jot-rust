#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{menu::{Menu, MenuItem, PredefinedMenuItem}, tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent}, Manager, State};
use tauri_plugin_global_shortcut::{Builder as GlobalShortcutBuilder, ShortcutState};
use chrono::Local;

#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER};

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

pub struct AppState {
    pub db: Mutex<Connection>,
}

impl AppState {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        let data_dir = app_handle.path().app_data_dir().expect("Failed to get app data dir");
        std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
        
        let db_path = data_dir.join("jot.db");
        let conn = Connection::open(&db_path).expect("Failed to create database");
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT,
                updated_at DATETIME
            )",
            [],
        ).expect("Failed to create notes table");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT
            )",
            [],
        ).expect("Failed to create settings table");

        AppState {
            db: Mutex::new(conn),
        }
    }
}

#[tauri::command]
fn list_notes(state: State<AppState>) -> Result<Vec<Note>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare("SELECT id, title, content, updated_at FROM notes ORDER BY updated_at DESC")
        .map_err(|e| e.to_string())?;
    
    let notes = stmt.query_map([], |row| {
        Ok(Note {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            updated_at: row.get(3)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<Note>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(notes)
}

#[tauri::command]
fn save_note(state: State<AppState>, id: String, title: String, content: String) -> Result<serde_json::Value, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    db.execute(
        "INSERT INTO notes (id, title, content, updated_at) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET title = excluded.title, content = excluded.content, updated_at = excluded.updated_at",
        params![id, title, content, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
fn delete_note(state: State<AppState>, id: String) -> Result<serde_json::Value, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    db.execute("DELETE FROM notes WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> Result<std::collections::HashMap<String, String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;
    
    let settings = stmt.query_map([], |row| {
        Ok(Setting {
            key: row.get(0)?,
            value: row.get(1)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<Setting>, _>>()
    .map_err(|e| e.to_string())?;

    let mut map = std::collections::HashMap::new();
    for setting in settings {
        map.insert(setting.key, setting.value);
    }

    Ok(map)
}

#[tauri::command]
fn save_setting(state: State<AppState>, key: String, value: String) -> Result<serde_json::Value, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    db.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )
    .map_err(|e| e.to_string())?;

    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
fn get_storage_path(app: tauri::AppHandle) -> Result<String, String> {
    let path = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

fn cursor_aligned_window_position(app: &tauri::AppHandle, width: i32, height: i32) -> Option<(i32, i32)> {
    let cursor = app.cursor_position().ok()?;
    let mut x = cursor.x.round() as i32 - width / 2;
    let mut y = cursor.y.round() as i32 - 28;

    if let Ok(monitors) = app.available_monitors() {
        if let Some(monitor) = monitors.into_iter().find(|monitor| {
            let area = monitor.work_area();
            cursor.x >= area.position.x as f64
                && cursor.x <= (area.position.x + area.size.width as i32) as f64
                && cursor.y >= area.position.y as f64
                && cursor.y <= (area.position.y + area.size.height as i32) as f64
        }) {
            let area = monitor.work_area();
            let min_x = area.position.x;
            let min_y = area.position.y;
            let max_x = (area.position.x + area.size.width as i32 - width).max(min_x);
            let max_y = (area.position.y + area.size.height as i32 - height).max(min_y);
            x = x.clamp(min_x, max_x);
            y = y.clamp(min_y, max_y);
        }
    }

    Some((x, y))
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if let Ok(size) = window.outer_size() {
            if let Some((x, y)) = cursor_aligned_window_position(app, size.width as i32, size.height as i32) {
                let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
            }
        }
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        // 确保窗口获得焦点（防止 hide() 后 show() 不生效的问题）
        let _ = window.set_always_on_top(true);
        let _ = window.set_always_on_top(false);
    }
}

fn toggle_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let is_visible = window.is_visible().unwrap_or(false);
        let is_minimized = window.is_minimized().unwrap_or(false);
        let is_focused = window.is_focused().unwrap_or(false);
        if is_visible && !is_minimized && is_focused {
            let _ = window.hide();
        } else {
            show_main_window(app);
        }
    }
}

fn setup_tray(app: &mut tauri::App) -> tauri::Result<()> {
    let show_item = MenuItem::with_id(app, "show", "显示 Jot", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &separator, &quit_item])?;

    let mut tray_builder = TrayIconBuilder::with_id("main-tray")
        .tooltip("Jot")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                toggle_main_window(tray.app_handle());
            }
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => show_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        });

    if let Some(icon) = app.default_window_icon().cloned() {
        tray_builder = tray_builder.icon(icon);
    }

    tray_builder.build(app)?;
    Ok(())
}

#[tauri::command]
fn minimize_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.minimize().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn maximize_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_maximized().unwrap_or(false) {
            window.unmaximize().map_err(|e| e.to_string())?;
        } else {
            window.maximize().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
fn close_window(app: tauri::AppHandle) -> Result<(), String> {
    // 根据 closeAction 设置决定行为：隐藏到托盘或退出应用
    let close_action = {
        let state = app.state::<AppState>();
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let result: Result<String, _> = db.query_row(
            "SELECT value FROM settings WHERE key = 'closeAction'",
            [],
            |row| row.get(0),
        );
        result.unwrap_or_else(|_| "hide".to_string())
    };

    if close_action == "quit" {
        app.exit(0);
    } else {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.hide();
        }
    }
    Ok(())
}

#[tauri::command]
fn is_window_maximized(app: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app.get_webview_window("main") {
        return window.is_maximized().map_err(|e| e.to_string());
    }
    Ok(false)
}

#[tauri::command]
fn resize_window(app: tauri::AppHandle, width: f64, height: f64, x: f64, y: f64) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        #[cfg(windows)]
        {
            let scale_factor = window.scale_factor().map_err(|e| e.to_string())?;
            let position = tauri::LogicalPosition::new(x, y).to_physical::<i32>(scale_factor);
            let size = tauri::LogicalSize::new(width, height).to_physical::<i32>(scale_factor);
            let hwnd = window.hwnd().map_err(|e| e.to_string())?;

            unsafe {
                SetWindowPos(
                    hwnd,
                    None,
                    position.x,
                    position.y,
                    size.width,
                    size.height,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                )
                .map_err(|e| e.to_string())?;
            }
        }

        #[cfg(not(windows))]
        {
            window.set_position(tauri::Position::Logical(tauri::LogicalPosition {
                x: x,
                y: y,
            })).map_err(|e| e.to_string())?;

        window.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: width,
            height: height,
        })).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
fn get_note(state: State<AppState>, id: String) -> Result<Option<Note>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db.prepare("SELECT id, title, content, updated_at FROM notes WHERE id = ?1")
        .map_err(|e| e.to_string())?;
    
    let mut notes = stmt.query_map(params![id], |row| {
        Ok(Note {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            updated_at: row.get(3)?,
        })
    })
    .map_err(|e| e.to_string())?
    .collect::<Result<Vec<Note>, _>>()
    .map_err(|e| e.to_string())?;

    Ok(notes.pop())
}

#[tauri::command]
fn set_always_on_top(app: tauri::AppHandle, always_on_top: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.set_always_on_top(always_on_top).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn hide_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn show_window(app: tauri::AppHandle) -> Result<(), String> {
    show_main_window(&app);
    Ok(())
}

#[tauri::command]
fn set_auto_launch(app: tauri::AppHandle, enable: bool) -> Result<(), String> {
    // 保存到设置中
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES ('auto_launch', ?1)",
        params![enable.to_string()],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_auto_launch(app: tauri::AppHandle) -> Result<bool, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let result: Result<String, _> = db.query_row(
        "SELECT value FROM settings WHERE key = 'auto_launch'",
        [],
        |row| row.get(0),
    );
    match result {
        Ok(value) => Ok(value == "true"),
        Err(_) => Ok(false),
    }
}

fn main() {
    let global_shortcut_plugin = GlobalShortcutBuilder::new()
        .with_shortcut("CmdOrCtrl+J")
        .expect("failed to parse global shortcut CmdOrCtrl+J")
        .with_handler(|app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                toggle_main_window(app);
            }
        })
        .build();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(global_shortcut_plugin)
        .setup(|app| {
            app.manage(AppState::new(&app.handle()));

            setup_tray(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_notes,
            get_note,
            save_note,
            delete_note,
            get_settings,
            save_setting,
            get_storage_path,
            minimize_window,
            maximize_window,
            close_window,
            is_window_maximized,
            resize_window,
            set_always_on_top,
            hide_window,
            show_window,
            set_auto_launch,
            get_auto_launch
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
