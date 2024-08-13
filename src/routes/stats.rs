use std::sync::Arc;
use axum::{Extension, Router};
use axum::routing::get;
use maud::{html, Markup};
use crate::AppState;

pub fn router(extension: Extension<Arc<AppState>>) -> Router {
    Router::new()
        .route("/", get(stats))
        .layer(extension)
}

async fn stats() -> Markup {
    html! {
        "Hello stats world!!"
    }
}
