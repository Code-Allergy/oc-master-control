use axum::body::Body;
use axum::Json;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use maud::{html, Markup};
use rand::Rng;
use serde_json::Value;

const KEY_LENGTH: usize = 48;

// client sends auth key as json with key named key
// server sends back either a key with key key or an error with key error



pub fn request_auth_snippet() -> Markup {
    html! {
        div class="flex flex-col align-center" {
            div id="key-result" class="flex flex-col align-center" {}
            a class="btn btn-ghost" hx-get="/api/new" hx-target="#key-result" { "Generate" }
        }
    }
}

// can add controls and restraints here, will return result
pub async fn generate_auth_snippet() -> Markup {
    let api_key = create_api_key();

    // store in db, also we can restraint here or before the call to.

    html! {
        code class="text-center p-4" {
            (create_api_key())
        }
    }
}

pub async fn generate_api_snippet(Json(payload): Json<Value>) -> Response {
    let api_key = create_api_key();

    // store this key along with the authorization key if valid, otherwise we should respond with an
    // error

    (StatusCode::OK, api_key).into_response()
}



pub fn create_api_key() -> String {
    let mut rng = rand::thread_rng();
    let key: String = (0..KEY_LENGTH)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect();
    key
}

pub async fn api_sitemap() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Unimplemented")
}
