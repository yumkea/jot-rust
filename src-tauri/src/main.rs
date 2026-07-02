#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::{DateTime, Local, NaiveDateTime};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::SystemTime,
};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, State,
};
use tauri_plugin_global_shortcut::{
    Builder as GlobalShortcutBuilder, GlobalShortcut, ShortcutState,
};

#[cfg(windows)]
use windows::Win32::{
    Foundation::RECT,
    UI::WindowsAndMessaging::{
        GetClientRect, GetWindowRect, SetWindowPos, SWP_NOACTIVATE, SWP_NOZORDER,
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}

pub struct AppState {
    pub db: Mutex<Connection>,
    pub notes_dir: Mutex<PathBuf>,
    pub is_hidden: Mutex<bool>,
    pub is_pinned: Mutex<bool>,
}

impl AppState {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        let legacy_data_dir = app_handle
            .path()
            .app_data_dir()
            .expect("Failed to get app data dir");
        let app_dir = std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| legacy_data_dir.clone());
        let config_dir = app_dir.join(".config");
        std::fs::create_dir_all(&config_dir).expect("Failed to create config directory");

        let legacy_db_path = legacy_data_dir.join("jot.db");
        let db_path = config_dir.join("jot.db");
        if !db_path.exists() && legacy_db_path.exists() {
            if let Err(error) = std::fs::copy(&legacy_db_path, &db_path) {
                eprintln!("Failed to copy legacy database: {error}");
            }
        }

        let default_notes_dir = config_dir.join("notes");
        let conn = Connection::open(&db_path).expect("Failed to create database");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                content TEXT,
                updated_at DATETIME
            )",
            [],
        )
        .expect("Failed to create notes table");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT
            )",
            [],
        )
        .expect("Failed to create settings table");

        let notes_dir = resolve_notes_dir(&conn, &default_notes_dir)
            .expect("Failed to resolve notes directory");
        std::fs::create_dir_all(&notes_dir).expect("Failed to create notes directory");
        migrate_sqlite_notes_to_files(&conn, &notes_dir)
            .expect("Failed to migrate notes to markdown files");
        merge_existing_markdown_notes(&notes_dir, &config_dir, &legacy_data_dir)
            .expect("Failed to merge existing markdown notes");
        ensure_markdown_notes_have_frontmatter(&notes_dir)
            .expect("Failed to prepare markdown metadata");

        AppState {
            db: Mutex::new(conn),
            notes_dir: Mutex::new(notes_dir),
            is_hidden: Mutex::new(false),
            is_pinned: Mutex::new(false),
        }
    }
}

fn resolve_notes_dir(conn: &Connection, default_notes_dir: &Path) -> Result<PathBuf, String> {
    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM settings WHERE key = 'notesDirectory'",
        [],
        |row| row.get(0),
    );

    Ok(match result {
        Ok(value) if !value.trim().is_empty() => PathBuf::from(value),
        _ => default_notes_dir.to_path_buf(),
    })
}

fn sanitize_filename_part(input: &str) -> String {
    let sanitized: String = input
        .chars()
        .map(|ch| match ch {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            ch if ch.is_control() => '_',
            ch => ch,
        })
        .collect();

    let trimmed = sanitized.trim().trim_matches('.').to_string();
    if trimmed.is_empty() {
        "Untitled".to_string()
    } else {
        trimmed.chars().take(80).collect()
    }
}

fn note_filename(id: &str, title: &str) -> String {
    format!(
        "{}--{}.md",
        sanitize_filename_part(title),
        sanitize_filename_part(id)
    )
}

fn now_string() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn system_time_to_string(time: SystemTime) -> String {
    let datetime: DateTime<Local> = time.into();
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn normalize_timestamp(value: &str) -> Option<String> {
    let trimmed = value.trim().trim_matches('"').trim_matches('\'');
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(datetime) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S") {
        return Some(datetime.format("%Y-%m-%d %H:%M:%S").to_string());
    }

    if let Ok(datetime) = DateTime::parse_from_rfc3339(trimmed) {
        return Some(
            datetime
                .with_timezone(&Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        );
    }

    None
}

fn yaml_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn yaml_unquote(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        return trimmed[1..trimmed.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\\\", "\\");
    }
    trimmed.trim_matches('\'').to_string()
}

#[derive(Debug, Clone)]
struct NoteMetadata {
    id: String,
    title: String,
    created_at: String,
    updated_at: String,
}

#[derive(Debug, Clone)]
struct MarkdownNote {
    metadata: NoteMetadata,
    content: String,
}

fn parse_frontmatter(raw: &str) -> (std::collections::HashMap<String, String>, String) {
    let normalized = raw.trim_start_matches('\u{feff}').replace("\r\n", "\n");
    if !normalized.starts_with("---\n") {
        return (std::collections::HashMap::new(), normalized);
    }

    let Some(end_index) = normalized[4..].find("\n---") else {
        return (std::collections::HashMap::new(), normalized);
    };

    let frontmatter = &normalized[4..4 + end_index];
    let mut body_start = 4 + end_index + "\n---".len();
    if normalized[body_start..].starts_with('\n') {
        body_start += 1;
    }

    let mut map = std::collections::HashMap::new();
    for line in frontmatter.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        map.insert(key.trim().to_string(), yaml_unquote(value));
    }

    (map, normalized[body_start..].to_string())
}

fn metadata_from_file(path: &Path, fallback_id: &str, fallback_title: &str) -> NoteMetadata {
    let metadata = fs::metadata(path).ok();
    let updated_at = metadata
        .as_ref()
        .and_then(|metadata| metadata.modified().ok())
        .map(system_time_to_string)
        .unwrap_or_else(now_string);
    let created_at = metadata
        .as_ref()
        .and_then(|metadata| metadata.created().ok())
        .map(system_time_to_string)
        .unwrap_or_else(|| updated_at.clone());

    NoteMetadata {
        id: fallback_id.to_string(),
        title: fallback_title.to_string(),
        created_at,
        updated_at,
    }
}

fn format_markdown_note(metadata: &NoteMetadata, content: &str) -> String {
    format!(
        "---\nid: \"{}\"\ntitle: \"{}\"\ncreated_at: {}\nupdated_at: {}\n---\n{}",
        yaml_escape(&metadata.id),
        yaml_escape(&metadata.title),
        metadata.created_at,
        metadata.updated_at,
        content
    )
}

fn read_markdown_note(
    path: &Path,
    fallback_id: &str,
    fallback_title: &str,
) -> Result<MarkdownNote, String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (frontmatter, content) = parse_frontmatter(&raw);
    let file_metadata = metadata_from_file(path, fallback_id, fallback_title);
    let id = frontmatter
        .get("id")
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| file_metadata.id.clone());
    let title = frontmatter
        .get("title")
        .filter(|value| !value.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| file_metadata.title.clone());
    let created_at = frontmatter
        .get("created_at")
        .and_then(|value| normalize_timestamp(value))
        .unwrap_or_else(|| file_metadata.created_at.clone());
    let updated_at = frontmatter
        .get("updated_at")
        .and_then(|value| normalize_timestamp(value))
        .unwrap_or_else(|| file_metadata.updated_at.clone());

    Ok(MarkdownNote {
        metadata: NoteMetadata {
            id,
            title,
            created_at,
            updated_at,
        },
        content,
    })
}

fn filename_parts(path: &Path) -> Result<(String, String), String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Invalid note filename".to_string())?;
    let stem = file_name.strip_suffix(".md").unwrap_or(file_name);
    Ok(stem
        .rsplit_once("--")
        .map(|(title, id)| (title.to_string(), id.to_string()))
        .unwrap_or_else(|| (stem.to_string(), stem.to_string())))
}

fn ensure_markdown_notes_have_frontmatter(notes_dir: &Path) -> Result<(), String> {
    for entry in fs::read_dir(notes_dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let (title, id) = filename_parts(&path)?;
        let note = read_markdown_note(&path, &id, &title)?;
        let rendered = format_markdown_note(&note.metadata, &note.content);
        if raw.replace("\r\n", "\n") != rendered {
            fs::write(&path, rendered).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn note_path_for_id(notes_dir: &Path, id: &str) -> Result<Option<PathBuf>, String> {
    let suffix = format!("--{}.md", sanitize_filename_part(id));
    for entry in fs::read_dir(notes_dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if file_name == format!("{}.md", sanitize_filename_part(id)) || file_name.ends_with(&suffix)
        {
            return Ok(Some(path));
        }
    }

    Ok(None)
}

fn note_from_path(path: &Path) -> Result<Note, String> {
    let (title, id) = filename_parts(path)?;
    let note = read_markdown_note(path, &id, &title)?;

    Ok(Note {
        id: note.metadata.id,
        title: note.metadata.title,
        content: note.content,
        created_at: note.metadata.created_at,
        updated_at: note.metadata.updated_at,
    })
}

fn merge_existing_markdown_notes(
    notes_dir: &Path,
    config_dir: &Path,
    legacy_data_dir: &Path,
) -> Result<(), String> {
    let mut candidates = vec![legacy_data_dir.join("notes")];

    if let Some(exe_dir) = config_dir.parent() {
        candidates.push(exe_dir.join(".config").join("notes"));
        if let Some(target_dir) = exe_dir.parent() {
            candidates.push(target_dir.join("debug").join(".config").join("notes"));
            candidates.push(target_dir.join("release").join(".config").join("notes"));
        }
    }

    for source_dir in candidates {
        if !source_dir.is_dir() || source_dir == notes_dir {
            continue;
        }

        for entry in fs::read_dir(&source_dir).map_err(|e| e.to_string())? {
            let source_path = entry.map_err(|e| e.to_string())?.path();
            if !source_path.is_file()
                || source_path.extension().and_then(|ext| ext.to_str()) != Some("md")
            {
                continue;
            }

            let note = note_from_path(&source_path)?;
            if let Some(existing_path) = note_path_for_id(notes_dir, &note.id)? {
                let existing_note = note_from_path(&existing_path)?;
                if existing_note.content == note.content
                    && (existing_note.created_at != note.created_at
                        || existing_note.updated_at != note.updated_at)
                {
                    let metadata = NoteMetadata {
                        id: existing_note.id,
                        title: existing_note.title,
                        created_at: note.created_at,
                        updated_at: note.updated_at,
                    };
                    fs::write(
                        existing_path,
                        format_markdown_note(&metadata, &existing_note.content),
                    )
                    .map_err(|e| e.to_string())?;
                }
                continue;
            }

            let Some(file_name) = source_path.file_name() else {
                continue;
            };
            let mut target_path = notes_dir.join(file_name);
            if target_path.exists() {
                target_path = notes_dir.join(note_filename(&note.id, &note.title));
            }
            fs::copy(&source_path, target_path).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn migrate_sqlite_notes_to_files(conn: &Connection, notes_dir: &Path) -> Result<usize, String> {
    let has_created_at = table_has_column(conn, "notes", "created_at")?;
    let has_updated_at = table_has_column(conn, "notes", "updated_at")?;
    let created_expr = if has_created_at { "created_at" } else { "NULL" };
    let updated_expr = if has_updated_at { "updated_at" } else { "NULL" };
    let sql = format!("SELECT id, title, content, {created_expr}, {updated_expr} FROM notes");
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                row.get::<_, Option<String>>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })
        .map_err(|e| e.to_string())?;

    let mut migrated_count = 0;
    for row in rows {
        let (id, title, content, created_at, updated_at) = row.map_err(|e| e.to_string())?;
        if note_path_for_id(notes_dir, &id)?.is_none() {
            let updated_at = updated_at
                .as_deref()
                .and_then(normalize_timestamp)
                .unwrap_or_else(now_string);
            let created_at = created_at
                .as_deref()
                .and_then(normalize_timestamp)
                .unwrap_or_else(|| updated_at.clone());
            let metadata = NoteMetadata {
                id: id.clone(),
                title: title.clone(),
                created_at,
                updated_at,
            };
            fs::write(
                notes_dir.join(note_filename(&id, &title)),
                format_markdown_note(&metadata, &content),
            )
            .map_err(|e| e.to_string())?;
            migrated_count += 1;
        }
    }

    Ok(migrated_count)
}

fn table_has_column(conn: &Connection, table: &str, column: &str) -> Result<bool, String> {
    let mut stmt = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|e| e.to_string())?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| e.to_string())?;

    for name in columns {
        if name.map_err(|e| e.to_string())? == column {
            return Ok(true);
        }
    }

    Ok(false)
}

#[tauri::command]
fn list_notes(state: State<AppState>) -> Result<Vec<Note>, String> {
    let mut notes = Vec::new();
    let notes_dir = state.notes_dir.lock().map_err(|e| e.to_string())?.clone();
    for entry in fs::read_dir(&notes_dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            notes.push(note_from_path(&path)?);
        }
    }

    notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Ok(notes)
}

#[tauri::command]
fn save_note(
    state: State<AppState>,
    id: String,
    title: String,
    content: String,
) -> Result<serde_json::Value, String> {
    let notes_dir = state.notes_dir.lock().map_err(|e| e.to_string())?.clone();
    let target_path = notes_dir.join(note_filename(&id, &title));
    let existing_path = note_path_for_id(&notes_dir, &id)?;
    let created_at = existing_path
        .as_ref()
        .and_then(|path| {
            read_markdown_note(path, &id, &title)
                .ok()
                .map(|note| note.metadata.created_at)
        })
        .unwrap_or_else(now_string);
    let metadata = NoteMetadata {
        id: id.clone(),
        title: title.clone(),
        created_at,
        updated_at: now_string(),
    };
    fs::write(&target_path, format_markdown_note(&metadata, &content))
        .map_err(|e| e.to_string())?;
    if let Some(existing_path) = existing_path {
        if existing_path != target_path {
            fs::remove_file(existing_path).map_err(|e| e.to_string())?;
        }
    }

    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
fn delete_note(state: State<AppState>, id: String) -> Result<serde_json::Value, String> {
    let notes_dir = state.notes_dir.lock().map_err(|e| e.to_string())?.clone();
    if let Some(path) = note_path_for_id(&notes_dir, &id)? {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }

    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
fn get_settings(
    state: State<AppState>,
) -> Result<std::collections::HashMap<String, String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare("SELECT key, value FROM settings")
        .map_err(|e| e.to_string())?;

    let settings = stmt
        .query_map([], |row| {
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
fn save_setting(
    state: State<AppState>,
    key: String,
    value: String,
) -> Result<serde_json::Value, String> {
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
    let state = app.state::<AppState>();
    let path = state.notes_dir.lock().map_err(|e| e.to_string())?.clone();
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
fn set_notes_directory(state: State<AppState>, path: String) -> Result<serde_json::Value, String> {
    let new_dir = PathBuf::from(path.trim());
    if new_dir.as_os_str().is_empty() {
        return Err("Notes directory cannot be empty".to_string());
    }

    fs::create_dir_all(&new_dir).map_err(|e| e.to_string())?;

    let old_dir = state.notes_dir.lock().map_err(|e| e.to_string())?.clone();
    if old_dir != new_dir && old_dir.exists() {
        for entry in fs::read_dir(&old_dir).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }

            let Some(file_name) = path.file_name() else {
                continue;
            };
            let note = note_from_path(&path)?;
            if note_path_for_id(&new_dir, &note.id)?.is_some() {
                continue;
            }
            let target = new_dir.join(file_name);
            if !target.exists() {
                fs::copy(&path, target).map_err(|e| e.to_string())?;
            }
        }
    }

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES ('notesDirectory', ?1)",
            params![new_dir.to_string_lossy().to_string()],
        )
        .map_err(|e| e.to_string())?;
    }

    *state.notes_dir.lock().map_err(|e| e.to_string())? = new_dir.clone();

    Ok(serde_json::json!({
        "success": true,
        "path": new_dir.to_string_lossy().to_string()
    }))
}

#[tauri::command]
fn migrate_legacy_database(
    state: State<AppState>,
    db_path: String,
) -> Result<serde_json::Value, String> {
    let legacy_db_path = PathBuf::from(db_path.trim());
    if legacy_db_path.as_os_str().is_empty() {
        return Err("Database path cannot be empty".to_string());
    }
    if !legacy_db_path.is_file() {
        return Err("Selected database file does not exist".to_string());
    }

    let notes_dir = state.notes_dir.lock().map_err(|e| e.to_string())?.clone();
    let conn =
        Connection::open_with_flags(&legacy_db_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| e.to_string())?;
    let migrated_count = migrate_sqlite_notes_to_files(&conn, &notes_dir)?;

    Ok(serde_json::json!({
        "success": true,
        "migrated": migrated_count,
        "dbPath": legacy_db_path.to_string_lossy().to_string()
    }))
}

fn cursor_aligned_window_position(
    app: &tauri::AppHandle,
    width: i32,
    height: i32,
) -> Option<(i32, i32)> {
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
            if let Some((x, y)) =
                cursor_aligned_window_position(app, size.width as i32, size.height as i32)
            {
                let _ = window
                    .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
            }
        }
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        // 确保窗口获得焦点（防止 hide() 后 show() 不生效的问题）
        let state: tauri::State<AppState> = app.state::<AppState>();
        let is_pinned = *state.is_pinned.lock().unwrap();
        let _ = window.set_always_on_top(true);
        if !is_pinned {
            let _ = window.set_always_on_top(false);
        }

        // 更新隐藏状态
        let mut is_hidden = state.is_hidden.lock().unwrap();
        *is_hidden = false;
    }
}

fn toggle_main_window(app: &tauri::AppHandle) {
    let state: tauri::State<AppState> = app.state::<AppState>();
    let is_hidden = *state.is_hidden.lock().unwrap();

    // 检查窗口是否可见、未最小化、已聚焦
    let should_hide = if let Some(window) = app.get_webview_window("main") {
        let is_visible = window.is_visible().unwrap_or(false);
        let is_minimized = window.is_minimized().unwrap_or(false);
        is_visible && !is_minimized
    } else {
        false
    };

    if is_hidden || !should_hide {
        show_main_window(app);
    } else if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
        // 更新隐藏状态
        let mut hidden = state.is_hidden.lock().unwrap();
        *hidden = true;
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
            // 更新隐藏状态
            let state: tauri::State<AppState> = app.state::<AppState>();
            let mut is_hidden = state.is_hidden.lock().unwrap();
            *is_hidden = true;
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
fn resize_window(
    app: tauri::AppHandle,
    width: f64,
    height: f64,
    x: f64,
    y: f64,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_maximized().map_err(|e| e.to_string())?
            || window.is_fullscreen().map_err(|e| e.to_string())?
        {
            return Ok(());
        }

        #[cfg(windows)]
        {
            let scale_factor = window.scale_factor().map_err(|e| e.to_string())?;
            let position = tauri::LogicalPosition::new(x, y).to_physical::<i32>(scale_factor);
            let size = tauri::LogicalSize::new(width, height).to_physical::<i32>(scale_factor);
            let hwnd = window.hwnd().map_err(|e| e.to_string())?;
            let mut window_rect = RECT::default();
            let mut client_rect = RECT::default();

            unsafe {
                GetWindowRect(hwnd, &mut window_rect).map_err(|e| e.to_string())?;
                GetClientRect(hwnd, &mut client_rect).map_err(|e| e.to_string())?;
            }

            let frame_width = (window_rect.right - window_rect.left)
                - (client_rect.right - client_rect.left);
            let frame_height = (window_rect.bottom - window_rect.top)
                - (client_rect.bottom - client_rect.top);
            let outer_width = size.width + frame_width.max(0);
            let outer_height = size.height + frame_height.max(0);

            unsafe {
                SetWindowPos(
                    hwnd,
                    None,
                    position.x,
                    position.y,
                    outer_width,
                    outer_height,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                )
                .map_err(|e| e.to_string())?;
            }
        }

        #[cfg(not(windows))]
        {
            window
                .set_position(tauri::Position::Logical(tauri::LogicalPosition {
                    x: x,
                    y: y,
                }))
                .map_err(|e| e.to_string())?;

            window
                .set_size(tauri::Size::Logical(tauri::LogicalSize {
                    width: width,
                    height: height,
                }))
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
fn get_note(state: State<AppState>, id: String) -> Result<Option<Note>, String> {
    let notes_dir = state.notes_dir.lock().map_err(|e| e.to_string())?.clone();
    note_path_for_id(&notes_dir, &id)?
        .map(|path| note_from_path(&path))
        .transpose()
}

#[tauri::command]
fn set_always_on_top(app: tauri::AppHandle, always_on_top: bool) -> Result<(), String> {
    {
        let state: tauri::State<AppState> = app.state::<AppState>();
        let mut is_pinned = state.is_pinned.lock().map_err(|e| e.to_string())?;
        *is_pinned = always_on_top;
    }

    if let Some(window) = app.get_webview_window("main") {
        window
            .set_always_on_top(always_on_top)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn hide_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
        // 更新隐藏状态
        let state: tauri::State<AppState> = app.state::<AppState>();
        let mut is_hidden = state.is_hidden.lock().unwrap();
        *is_hidden = true;
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
    )
    .map_err(|e| e.to_string())?;
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

#[tauri::command]
fn update_global_shortcut(app: tauri::AppHandle, shortcut: String) -> Result<(), String> {
    register_show_window_shortcut(&app, &shortcut)
}

fn register_show_window_shortcut(app: &tauri::AppHandle, shortcut_str: &str) -> Result<(), String> {
    let gs = app.state::<GlobalShortcut<_>>();
    // 先注销旧的快捷键
    let _ = gs.unregister_all();

    // 将前端格式 (Ctrl+J) 转换为 Tauri 格式 (CmdOrCtrl+J)
    let tauri_shortcut = shortcut_str
        .replace("Ctrl+", "CmdOrCtrl+")
        .replace("Alt+", "Alt+")
        .replace("Shift+", "Shift+");

    gs.on_shortcut(tauri_shortcut.as_str(), move |app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            toggle_main_window(app);
        }
    })
    .map_err(|e| e.to_string())?;

    Ok(())
}

fn main() {
    let global_shortcut_plugin = GlobalShortcutBuilder::new().build();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(global_shortcut_plugin)
        .setup(|app| {
            app.manage(AppState::new(&app.handle()));

            // 从数据库读取 show_window 快捷键并注册
            let state = app.state::<AppState>();
            let shortcut_str = {
                let db = state.db.lock().unwrap();
                let result: Result<String, _> = db.query_row(
                    "SELECT value FROM settings WHERE key = 'show_window'",
                    [],
                    |row| row.get(0),
                );
                result.unwrap_or_else(|_| "Ctrl+J".to_string())
            };

            // 注册全局快捷键
            let gs = app.state::<GlobalShortcut<_>>();
            let tauri_shortcut = shortcut_str
                .replace("Ctrl+", "CmdOrCtrl+")
                .replace("Alt+", "Alt+")
                .replace("Shift+", "Shift+");

            let _ = gs.on_shortcut(tauri_shortcut.as_str(), move |app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    toggle_main_window(app);
                }
            });

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
            set_notes_directory,
            migrate_legacy_database,
            minimize_window,
            maximize_window,
            close_window,
            is_window_maximized,
            resize_window,
            set_always_on_top,
            hide_window,
            show_window,
            set_auto_launch,
            get_auto_launch,
            update_global_shortcut
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
