mod api;
mod hx_middleware;
mod auth;
mod schema;
mod models;

use std::env;
use http::StatusCode;
use std::time::Duration;
use axum::{middleware, Router};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum_htmx::HxRequest;
use diesel::{Connection, SqliteConnection};
use maud::{DOCTYPE, html, Markup, PreEscaped, Render};
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    trace::TraceLayer,
    timeout::TimeoutLayer
};
use tower_http::services::{ServeDir, ServeFile};
use crate::api::{create_api_key, generate_api_snippet, generate_auth_snippet, request_auth_snippet};
use crate::auth::auth_router;
use crate::hx_middleware::hx_response_middleware;


// DATABASE SCHEMAS
/*
    clients {
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


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = Router::new()
        .route("/", get(index))
        .route("/stats", get(stats))
        .route("/settings", get(settings))
        // todo add route handler for api calls
        .route("/api/new", get(generate_auth_snippet))
        .nest("/auth", auth_router())
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/favicon.ico", ServeFile::new("favicon.ico"))
        .fallback(page_not_found)
        .layer((
            ServiceBuilder::new()
                .layer(middleware::from_fn(hx_response_middleware))
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(Duration::from_secs(10))),
        ))

        ;

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));


    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Serving at localhost:3000!");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

}


async fn index(HxRequest(hx_request): HxRequest) -> Markup {
    html! {
        div class="container mx-auto px-4" {
            h1 .text-center {
                "Hello world!"
            }
            (client())
            (client())
            (client())
            (client())
            (client())
            (client())
            (client())
            (client())
        }


    }
}

fn client() -> Markup {
    html! {
        .flex.border."p-2" {
            .flex.flex-col."pr-4" {
                img src="/favicon.ico" {}
                p { "ONLINE" }
            }
            .flex.flex-col {
                p.font-bold { "AE System L1" }
                p."flex-1" { "Above crafting terminal" }
                p { "Monitor AE activity and act as display" }
            }
        }
    }
}

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
    assert!(!&BUTTONS.is_empty(), "Navigation bar should have buttons (none were loaded from nav_buttons())");

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
        (Modal::new("request_authorization_modal", "Generate a new authorization key:", request_auth_snippet()))
    }
}

struct Modal {
    id: &'static str,
    title: &'static str,
    body: Markup,
}

impl Modal {
    pub fn new(id: &'static str, title: &'static str, body: Markup) -> Modal {
        Modal {
            id, title, body
        }
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
    Link::new("Settings", "/settings")
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
        }
    )
}



// Make our own error that wraps `anyhow::Error`.
struct ServerError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
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



