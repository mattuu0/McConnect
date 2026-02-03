use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct TunnelHandle {
    pub join_handle: tokio::task::JoinHandle<()>,
    pub ping_tx: tokio::sync::mpsc::UnboundedSender<()>,
}

#[derive(Default)]
pub struct AppState {
    pub tunnels: HashMap<String, TunnelHandle>,
    pub server_handle: Option<tokio::task::JoinHandle<()>>,
}

pub static STATE: Lazy<Arc<Mutex<AppState>>> =
    Lazy::new(|| Arc::new(Mutex::new(AppState::default())));
