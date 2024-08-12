mod api;
mod auth;
mod hx_middleware;
mod models;
mod schema;

use std::collections::{HashMap, HashSet, VecDeque};
use self::models::*;
use crate::auth::auth_router;
use crate::hx_middleware::hx_response_middleware;
use crate::schema::clients::dsl::clients;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{middleware, Extension, Router};
use axum_htmx::HxRequest;
use diesel::prelude::*;
use diesel::Connection;
use http::StatusCode;
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};
use site::db;
use std::env;
use std::iter::Map;
use std::sync::Arc;
use std::time::Duration;
use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use futures_util::stream::SplitStream;
use thiserror::Error;
use tokio::signal;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tower::ServiceBuilder;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing::log::error;
use site::db::Pool;
use crate::api::request_auth_snippet;

const HEARTBEAT_DELAY: u64 = 30;

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

pub struct AppState {
    pub pool: Pool,
    pub active_clients: Arc<Mutex<HashMap<i64, ActiveClient>>>
}


#[tokio::main]
async fn main() {
    use crate::schema::clients;
    tracing_subscriber::fmt::init();
    dotenv::dotenv().expect("Failed to load .env");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut pool = db::establish_connection_pool(&database_url);

    let mut state = Arc::new(AppState {
        pool,
        active_clients: Arc::new(Mutex::new(HashMap::new())),
    });

    let app = Router::new()
        .route("/", get(index))
        .route("/clients", get(clients_page))
        .route("/stats", get(stats))
        .route("/settings", get(settings))
        // todo add route handler for api calls
        .nest("/auth", auth_router(Extension(state.clone())))
        .nest("/api", api::router(Extension(state.clone())))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/favicon.ico", ServeFile::new("favicon.ico"))
        .layer(Extension(state.clone()))
        .fallback(page_not_found)
        .layer(ServiceBuilder::new()
            .layer(middleware::from_fn(hx_response_middleware))
            .layer(TraceLayer::new_for_http())
            .layer(TimeoutLayer::new(Duration::from_secs(10)))
        );


    tokio::spawn(client_heartbeat(state));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("Serving at 0.0.0.0:3000!");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn client_heartbeat(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(Duration::from_secs(HEARTBEAT_DELAY));
    loop {
        interval.tick().await; // Wait for the 30-second interval
        let mut all_active_clients = state.active_clients.lock().await;

        let mut to_remove = HashSet::new();

        for mut active_client in all_active_clients.values_mut() {
            println!("Sent id");
            let client_id = *active_client.client.id();
            active_client.sender.send(Message::Ping(Vec::new())).await.unwrap_or_else(|_| {
                to_remove.insert(client_id);
            });
        }

        // // read all off stack
        sleep(Duration::from_secs(3)).await;
        'responded: for mut active_client in all_active_clients.values_mut() {
            let mut message_queue = active_client.message_queue.lock().await;
            println!("MSG: {}", message_queue.len());
            let client_id = *active_client.client.id();
            for _ in 0..message_queue.len() {
                let message = message_queue.pop_front().unwrap();
                if let Message::Pong(_) = message { continue 'responded; }
                message_queue.push_back(message);
            }

            to_remove.insert(client_id);
        }

        for exited_client_id in to_remove {
            let mut client = all_active_clients.remove(&exited_client_id).unwrap();
            client.sender.close().await.expect("Failed to close socket from exited client");
            client.thread_handle.await.unwrap()
        };
    }

}




async fn clients_page(Extension(state): Extension<Arc<AppState>>) -> Result<Markup, ServerError> {
    let mut conn = state.pool.get()?;
    let received_clients = Client::get_all(&mut conn)?;

    Ok(html! {
        @for client in received_clients {
            (MiniClient::from_client(client))
        }
    })
}

async fn index() -> Markup {
    html! {
        div class="container mx-auto px-4" {
            h1 .text-center {
                "Hello world!"
            }
            "Result: "
        }
    }
}

// fn client() -> Markup {
//
// }

// root page layout
pub(crate) fn root(contents: PreEscaped<String>) -> String {
    let html_res = html! {
        (head())
        (navbar())
        #body-contents {
            (contents)
        }
        (footer())
    };

    html_res.into_string()
}
pub fn head() -> Markup {
    html! {
            (DOCTYPE)
            html lang="en";
            head {
                meta charset="utf-8";
                link rel="stylesheet" type="text/css" href="/static/style.css";
                title { "OpenComputer Access Terminal" }
            }
    }
}
pub fn footer() -> Markup {
    html! {
        script src="https://unpkg.com/htmx.org@1.9.12" {}
        script src="/static/script.js" {}
    }
}
pub(crate) fn navbar() -> Markup {
    assert!(
        !&BUTTONS.is_empty(),
        "Navigation bar should have buttons (none were loaded from nav_buttons())"
    );

    let last_button = &BUTTONS.last().unwrap();
    html! {
        div ."navbar bg-base-100" hx-boost="true" hx-target="#body-contents" hx-push-url="true" {
            // some icon here instead..
            div ."flex-none" {
                button ."btn btn-square btn-ghost" {
                    svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        class="inline-block h-5 w-5 stroke-current" {
                        path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M4 6h16M4 12h16M4 18h16" {}
                    }
                }
            }

            @for button in &BUTTONS[0 .. &BUTTONS.len()-1] {
                a ."btn btn-ghost text-xl"
                    href=(button.url) { (button.name) }
            }
            div ."flex-1" {
                a ."btn btn-ghost text-xl" hx-target="#body-contents" hx-push-url="true"
                    href=(last_button.url) { (last_button.name) }
            }

            div ."flex items-stretch" {
                button class="btn btn-info rounded-btn" onclick={"request_authorization_modal.showModal()"} { "+" }
                div ."dropdown dropdown-end" {
                    div tabindex="0" role="button" class="btn btn-ghost rounded-btn" {
                        svg
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            class="inline-block h-5 w-5 stroke-current" {
                            path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M5 12h.01M12 12h.01M19 12h.01M6 12a1 1 0 11-2 \
                                0 1 1 0 012 0zm7 0a1 1 0 11-2 0 1 1 0 012 0zm7 0a1 \
                                1 0 11-2 0 1 1 0 012 0z" {}
                        }
                    }

                    ul tabindex="0" class="menu \
                    dropdown-content bg-base-100 rounded-box z-[1] mt-4 w-52 p-2 shadow" {
                        @for button in &DROPDOWN_BUTTONS {
                            li { a hx-get=(button.url) { (button.name) } }
                        }

                    }
                }
            }
        }
        (Modal::new("request_authorization_modal",
            "Generate a new authorization key:",
            request_auth_snippet()))
    }
}

struct Modal {
    id: &'static str,
    title: &'static str,
    body: Markup,
}

impl Modal {
    pub fn new(id: &'static str, title: &'static str, body: Markup) -> Modal {
        Modal { id, title, body }
    }
}

impl Render for Modal {
    fn render(&self) -> Markup {
        html! {
            dialog id=(self.id) class="modal" {
                div class="modal-box" {
                    h3 class="text-lg font-bold text-center" { (self.title) }
                    p class="py-4" { (self.body) }
                }

                form method="dialog" class="modal-backdrop" {
                    button { "close" }
                }
            }
        }
    }
}

pub struct Link {
    pub(crate) name: &'static str,
    pub(crate) url: &'static str,
}

impl Link {
    pub const fn new(name: &'static str, url: &'static str) -> Link {
        Link { name, url }
    }
}

pub static BUTTONS: [Link; 5] = [
    Link::new("Home", "/"),
    Link::new("Clients", "/clients"),
    Link::new("Energy", "/energy"),
    Link::new("Items", "/items"),
    Link::new("Statistics", "/stats"),
];

pub static DROPDOWN_BUTTONS: [Link; 2] = [
    Link::new("Unknown", "/seals"),
    Link::new("Settings", "/settings"),
];

async fn shutdown_signal() {
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

async fn page_not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        html! {
            "The following page was not found!"
        },
    )
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Internal Server Error: {0}")]
    Internal(anyhow::Error),

    #[error("Service Unavailable: {0}")]
    ServiceUnavailable(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest(msg) => {
                (
                    StatusCode::BAD_REQUEST,
                    format!("Bad Request: {}", msg),
                )
                    .into_response()
            }
            AppError::Unauthorized(msg) => {
                (
                    StatusCode::UNAUTHORIZED,
                    format!("Unauthorized: {}", msg),
                )
                    .into_response()
            }
            AppError::Forbidden(msg) => {
                (
                    StatusCode::FORBIDDEN,
                    format!("Forbidden: {}", msg),
                )
                    .into_response()
            }
            AppError::NotFound(msg) => {
                (
                    StatusCode::NOT_FOUND,
                    format!("Not Found: {}", msg),
                )
                    .into_response()
            }
            AppError::Internal(err) => {
                error!("Internal error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Internal Server Error: {}", err),
                )
                    .into_response()
            }
            AppError::ServiceUnavailable(msg) => {
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    format!("Service Unavailable: {}", msg),
                )
                    .into_response()
            }
        }
    }
}

impl AppError {
    // Helper function to create an Internal error from any anyhow::Error
    pub fn internal<E>(err: E) -> Self
    where
        E: Into<anyhow::Error>
    {
        AppError::Internal(err.into())
    }
}


// Make our own error that wraps `anyhow::Error`.
struct ServerError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        error!("Server error: {}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

async fn stats() -> Markup {
    html! {
        "Hello other world!!"
    }
}
async fn settings() -> Markup {
    html! {
        "Hello settings!!"
    }
}
