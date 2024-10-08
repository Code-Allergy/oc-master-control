use std::sync::Arc;
use std::time::Duration;
use axum::{middleware, Extension, Router};
use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use maud::{html, Markup, PreEscaped};
use axum::routing::get;
use http::header::CONTENT_LENGTH;
use tower::ServiceBuilder;
use include_dir::{Dir, include_dir};
use tower_serve_static::{include_file, ServeDir, ServeFile};

use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use crate::AppState;
use crate::error::page_not_found;
use crate::layout::root;

pub mod api;
pub mod stats;
pub mod auth;
pub mod client_routes;

static ASSETS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/static");


pub fn router(state: Arc<AppState>) -> Router {
    
    Router::new()
        /*  simple routes  */
        .route("/", get(index))
        .route("/settings", get(settings))

        /*  nested routes  */
        .nest("/api", api::router(Extension(state.clone())))
        .nest("/auth", auth::router(Extension(state.clone())))
        .nest("/clients", client_routes::router(Extension(state.clone())))
        .nest("/stats", stats::router(Extension(state.clone())))

        /*  Nested services  */
        .nest_service("/static", ServeDir::new(&ASSETS_DIR))
        .nest_service("/favicon.ico", ServeFile::new(include_file!("/favicon.ico")))
        .fallback(page_not_found)
        .layer(ServiceBuilder::new()
            .layer(Extension(state.clone()))
            .layer(middleware::from_fn(hx_response_middleware))
            .layer(TraceLayer::new_for_http())
            .layer(TimeoutLayer::new(Duration::from_secs(10)))
        )
}


/// Proper handling of htmx and non-htmx requests
pub async fn hx_response_middleware(request: Request<Body>, next: Next) -> Response {
    let request_uri = request.uri();
    let is_htmx = request
        .headers()
        .get("HX-Request")
        .is_some_and(|h| h.to_str().is_ok_and(|v| v == "true"));

    let is_static = request_uri.to_string().eq("/favicon.ico")
        || request_uri.to_string().starts_with("/static");

    let is_api = request_uri.to_string().starts_with("/api");
    let response = next.run(request).await;

    /* Render standard html on non-hx requests & static/api routes */
    if !( !is_htmx && !is_static && !is_api ) { response } else {
        let (response_parts, response_body) = response.into_parts();
        let body_bytes = axum::body::to_bytes(response_body, usize::MAX)
            .await
            .unwrap_or_default();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap_or_default();
        let new_body = root(PreEscaped(body_str));
        let new_len = new_body.len();

        // todo note that this strips off any existing headers
        Response::builder()
            .status(response_parts.status)
            .header(CONTENT_LENGTH, new_len)
            .body(Body::new(new_body))
            .unwrap()
    }
}

/// /
async fn index() -> Markup {
    html! {
        div class="w-full max-w-3xl h-96 p-4 bg-gray-900 text-white rounded-lg shadow-md border border-gray-700 overflow-auto"
            id="terminal"
            hx-get="/load-more"
            hx-trigger="scroll"
            hx-swap="beforeend"
            hx-target="#terminal" {
            div id="content" {
                p { "Welcome to the server terminal!" }
                p { "Loading logs..." }
                p { "Loading logs..." }
                p { "Loading logs..." }
                p { "Loading logs..." }
                p { "Loading logs..." }
                p { "Loading logs..." }
                p { "Loading logs..." }
                p { "Loading logs..." }
                p { "Loading logs..." }
                
            }
        }
    }
}

/// /settings
async fn settings() -> Markup {
    html! {
        "Hello settings!!"
    }
}
