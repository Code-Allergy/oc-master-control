use axum::body::Body;
use axum::{Form, Json};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use maud::{html, Markup};
use rand::Rng;
use serde::Deserialize;
use serde_json::Value;

const KEY_LENGTH: usize = 48;
const AUTHORIZATION_KEY_LENGTH: usize = 8;
// client sends auth key as json with key named key
// server sends back either a key with key key or an error with key error



pub fn request_auth_snippet() -> Markup {
    html! {
        div class="flex flex-col align-center" {
            div id="key-result" class="flex flex-col align-center" {}
            form .flex.flex-col hx-post="/api/new" hx-target="#key-result" {
                input name="name" type="text" placeholder="Client Name" .input.input-bordered.input-primary.w-full.max-w-xs;
                button class="btn btn-ghost" action="submit" { "Generate" }
            }
        }
    }
}


#[derive(Deserialize)]
pub struct AuthFormData {
    name: String,
}

// can add controls and restraints here, will return result
pub async fn generate_auth_snippet(Form(data): Form<AuthFormData>) -> Markup {
    // store in db, also we can restraint here or before the call to.

    html! {
        code class="text-center p-4" {
            (create_api_key(AUTHORIZATION_KEY_LENGTH))
        }
    }
}

pub async fn generate_api_snippet(Json(payload): Json<Value>) -> Response {
    let api_key = create_api_key(KEY_LENGTH);

    // store this key along with the authorization key if valid, otherwise we should respond with an
    // error

    (StatusCode::OK, api_key).into_response()
}



pub fn create_api_key(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let key: String = (0..length)
        .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
        .collect();
    key
}

pub async fn api_sitemap() -> impl IntoResponse {
    (StatusCode::NOT_IMPLEMENTED, "Unimplemented")
}
