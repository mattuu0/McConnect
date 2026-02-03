use crate::models::LogEntry;
use chrono::Local;
use log::{Level, Metadata, Record};
use once_cell::sync::OnceCell;
use tauri::{AppHandle, Emitter, Runtime};

static APP_HANDLE: OnceCell<AppHandle<tauri::Wry>> = OnceCell::new();

pub struct TauriLogger;

impl log::Log for TauriLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let timestamp = Local::now().format("%H:%M:%S").to_string();
            let message = record.args().to_string();
            let level = record.level().to_string().to_uppercase();

            // ターミナルにも出力 (tauri dev 用)
            eprintln!("[{}] {}: {}", timestamp, level, message);

            if let Some(app) = APP_HANDLE.get() {
                let _ = app.emit(
                    "log-event",
                    LogEntry {
                        timestamp,
                        level,
                        message,
                    },
                );
            }
        }
    }

    fn flush(&self) {}
}

static LOGGER: TauriLogger = TauriLogger;

pub fn init_logger(app_handle: AppHandle<tauri::Wry>) {
    let _ = APP_HANDLE.set(app_handle);
    let _ = log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::Info));
}

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
