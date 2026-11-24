#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rusqlite::{params, Connection};
use serde::Serialize;
use std::fs;
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
struct Task {
    id: i64,
    title: String,
    project: String,
    priority: String,
    estimate_minutes: i64,
    status: String,
    planned_date: String,
    created_at: String,
    completed_at: Option<String>,
}

#[derive(Serialize)]
struct DailySummary {
    total: i64,
    done: i64,
    focus_minutes: i64,
}

fn db_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("fokusflow.db"))
}

fn db(app: &AppHandle) -> Result<Connection, String> {
    let conn = Connection::open(db_path(app)?).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            project TEXT NOT NULL DEFAULT 'Pribadi',
            priority TEXT NOT NULL DEFAULT 'normal',
            estimate_minutes INTEGER NOT NULL DEFAULT 25,
            status TEXT NOT NULL DEFAULT 'open',
            planned_date TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            completed_at TEXT
        );
        CREATE TABLE IF NOT EXISTS focus_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER,
            started_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            minutes INTEGER NOT NULL,
            FOREIGN KEY(task_id) REFERENCES tasks(id)
        );
        CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            content TEXT NOT NULL DEFAULT '',
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        INSERT OR IGNORE INTO notes(id, content) VALUES (1, '');"
    ).map_err(|e| e.to_string())?;
    Ok(conn)
}

#[tauri::command]
fn list_tasks(app: AppHandle, date: String) -> Result<Vec<Task>, String> {
    let conn = db(&app)?;
    let mut stmt = conn.prepare(
        "SELECT id, title, project, priority, estimate_minutes, status, planned_date, created_at, completed_at
         FROM tasks WHERE planned_date = ? ORDER BY status, CASE priority WHEN 'high' THEN 1 WHEN 'normal' THEN 2 ELSE 3 END, id DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(params![date], |r| Ok(Task {
        id: r.get(0)?, title: r.get(1)?, project: r.get(2)?, priority: r.get(3)?,
        estimate_minutes: r.get(4)?, status: r.get(5)?, planned_date: r.get(6)?,
        created_at: r.get(7)?, completed_at: r.get(8)?,
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
fn add_task(app: AppHandle, title: String, project: String, priority: String, estimate_minutes: i64, planned_date: String) -> Result<(), String> {
    db(&app)?.execute(
        "INSERT INTO tasks(title, project, priority, estimate_minutes, planned_date) VALUES (?, ?, ?, ?, ?)",
        params![title, project, priority, estimate_minutes, planned_date]
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn toggle_task(app: AppHandle, id: i64, done: bool) -> Result<(), String> {
    let status = if done { "done" } else { "open" };
    db(&app)?.execute(
        "UPDATE tasks SET status = ?, completed_at = CASE WHEN ? = 'done' THEN CURRENT_TIMESTAMP ELSE NULL END WHERE id = ?",
        params![status, status, id]
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn save_focus_session(app: AppHandle, task_id: Option<i64>, minutes: i64) -> Result<(), String> {
    db(&app)?.execute("INSERT INTO focus_sessions(task_id, minutes) VALUES (?, ?)", params![task_id, minutes])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn daily_summary(app: AppHandle, date: String) -> Result<DailySummary, String> {
    let conn = db(&app)?;
    let (total, done): (i64, i64) = conn.query_row(
        "SELECT COUNT(*), SUM(CASE WHEN status = 'done' THEN 1 ELSE 0 END) FROM tasks WHERE planned_date = ?", params![date],
        |r| Ok((r.get(0)?, r.get::<_, Option<i64>>(1)?.unwrap_or(0)))
    ).map_err(|e| e.to_string())?;
    let focus_minutes: i64 = conn.query_row(
        "SELECT COALESCE(SUM(minutes), 0) FROM focus_sessions WHERE date(started_at, 'localtime') = ?", params![date], |r| r.get(0)
    ).map_err(|e| e.to_string())?;
    Ok(DailySummary { total, done, focus_minutes })
}

#[tauri::command]
fn load_note(app: AppHandle) -> Result<String, String> {
    db(&app)?.query_row("SELECT content FROM notes WHERE id = 1", [], |r| r.get(0)).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_note(app: AppHandle, content: String) -> Result<(), String> {
    db(&app)?.execute("UPDATE notes SET content = ?, updated_at = CURRENT_TIMESTAMP WHERE id = 1", params![content])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn database_location(app: AppHandle) -> Result<String, String> {
    Ok(db_path(&app)?.display().to_string())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| { db(&app.handle()).map_err(|e| std::io::Error::other(e))?; Ok(()) })
        .invoke_handler(tauri::generate_handler![list_tasks, add_task, toggle_task, save_focus_session, daily_summary, load_note, save_note, database_location])
        .run(tauri::generate_context!())
        .expect("FokusFlow tidak dapat dijalankan");
}
