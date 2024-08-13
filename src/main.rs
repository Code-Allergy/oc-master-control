use std::collections::HashMap;
use site::routes::auth::auth_router;
use axum::routing::get;
use axum::{middleware, Extension, Router};
use maud::{html, Markup, PreEscaped};
use site::{database, shutdown_signal, AppState};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use http::header::CONTENT_LENGTH;
use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use site::error::page_not_found;
use site::layout::root;
use site::routes::{api, client_routes, stats};

const SERVER_ADDR: &str = "0.0.0.0:3000";

// DATABASE SCHEMAS
/*
   clients {
       id
       user_id
       authorization key
       api key
       time of enrollment
       time of last response
       name: string
       status: status enum
       revoked: boolean
   }

   ae_history {

   }

   power_history

   .. other tables of history


   status enum { (text in db)
       Authorized
       Enrolled
       Connected
   }

   * with this design, we save everything to database, then wipe the Authorized (unenrolled) keys
   after some set period

*/

// primary todos:
//
// working errors for other types, and possibly able to pass in error code in the text,
// enum for main AppError, can have other type errors automatically translated and returned.

// websocket handler and command/control portion of website/client
//


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().expect("Failed to load .env"); // TODO TMP DEV-REMOVE-ME

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let state = Arc::new(AppState {
        pool: database::establish_connection_pool(&database_url),
        active_clients: Arc::new(Mutex::new(HashMap::new())),
    });

    let app = Router::new()
        /*  simple routes  */
        .route("/", get(index))
        .route("/settings", get(settings))

        /*  nested routes  */
        .nest("/api", api::router(Extension(state.clone())))
        .nest("/auth", auth_router(Extension(state.clone())))
        .nest("/clients", client_routes::router(Extension(state.clone())))
        .nest("/stats", stats::router(Extension(state.clone())))

        /*  Nested services  */
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/favicon.ico", ServeFile::new("favicon.ico"))
        .fallback(page_not_found)
        .layer(ServiceBuilder::new()
            .layer(Extension(state.clone()))
            .layer(middleware::from_fn(hx_response_middleware))
            .layer(TraceLayer::new_for_http())
            .layer(TimeoutLayer::new(Duration::from_secs(10)))
        );

    let listener = tokio::net::TcpListener::bind(SERVER_ADDR).await.unwrap();

    println!("Serving at {SERVER_ADDR}!");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

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

async fn index() -> Markup {
    html! {
        div class="container mx-auto px-4" {
            h1 .text-center {
                "Hello world!"
            }
        }
    }
}

async fn settings() -> Markup {
    html! {
        "Hello settings!!"
    }
}
