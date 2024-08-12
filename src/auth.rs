use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use http::StatusCode;
use maud::{html, Markup};

pub fn auth_router() -> Router {
    Router::new()
        .route("/register", get(login))
        .route("/authorized", get(login))
        .route("/login", get(login))
        .route("/logout", get(login))
}

async fn login() -> Markup {
    html! {
        "Login here!"
    }
}
