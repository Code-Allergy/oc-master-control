use std::sync::Arc;
use axum::routing::get;
use axum::{Extension, Router};
use maud::{html, Markup};
use crate::AppState;

pub fn router(state: Extension<Arc<AppState>>) -> Router {
    Router::new()
        .route("/register", get(login))
        .route("/authorized", get(login))
        .route("/login", get(login))
        .route("/logout", get(login))
        .layer(Extension(state))
}

async fn login() -> Markup {
    html! {
        "Login here!"
    }
}
