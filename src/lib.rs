use std::collections::HashMap;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::Mutex;
use database::models::ActiveClient;
use crate::database::Pool;

pub mod error;
pub mod layout;
pub mod routes;
pub mod database;

/// Primary app state engine
pub struct AppState {
    pub pool: Pool,
    pub active_clients: Arc<Mutex<HashMap<i64, ActiveClient>>>
}


pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}


