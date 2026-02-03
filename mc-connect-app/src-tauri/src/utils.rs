use crate::models::LogEntry;
use chrono::Local;
use tauri::{AppHandle, Emitter, Runtime};

pub fn emit_log<R: Runtime>(app: &AppHandle<R>, level: &str, message: String) {
    let timestamp = Local::now().format("%H:%M:%S").to_string();
    let _ = app.emit(
        "log-event",
        LogEntry {
            timestamp,
            level: level.to_string(),
            message,
        },
    );
}
