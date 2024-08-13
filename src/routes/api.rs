use std::collections::VecDeque;
use std::sync::Arc;
use crate::database::models::{ActiveClient, Client, NewClient, NewClientLog};
use crate::{AppState};
use anyhow::anyhow;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::WebSocketUpgrade;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Extension, Form, Json, Router};
use diesel::prelude::*;
use diesel::{RunQueryDsl, SelectableHelper};
use http::{HeaderMap, StatusCode};
use maud::{html, Markup};
use rand::Rng;
use serde::Deserialize;
use serde_json::Value;
use time::{OffsetDateTime, PrimitiveDateTime};
use futures_util::{stream::{StreamExt, SplitStream}};
use phf::phf_map;
use tokio::sync::Mutex;
use crate::database::schema::client_logs::dsl::client_logs;
use crate::error::ServerError;

const KEY_LENGTH: usize = 48;
const AUTHORIZATION_KEY_LENGTH: usize = 8;
// client sends auth key as json with key named key
// server sends back either a key with key key or an error with key error


// commands issued BY the server, directed at one or many clients
pub enum ServerWSCommand {
    Update, About, Response,
}

static SERVER_WS_COMMAND_STRINGS: phf::Map<&'static str, ServerWSCommand> = phf_map! {

};

static CLIENT_WS_COMMAND_STRINGS: phf::Map<&'static str, ClientWSCommand> = phf_map! {
    "Log" => ClientWSCommand::Log(None)
};

impl ServerWSCommand {
    // pub fn from_str(string: &str) -> Result<ServerWSCommand, ServerError> {
    //     let command = if let Some((cmd_str, data)) = string.split_once(" ") {
    //         let cmd = SERVER_WS_COMMAND_STRINGS.get(cmd_str).ok_or(anyhow!("Invalid command."))?;
    //     } else {
    //         SERVER_WS_COMMAND_STRINGS.get(string).ok_or(anyhow!("Invalid command."))?;
    //     };
    // }
    pub fn as_str(&self) -> &'static str {
        match self {
            ServerWSCommand::Update => "Update",
            ServerWSCommand::About => "About",
            ServerWSCommand::Response => "Response",
        }
    }
}

// commands issued by the client, directed at the server
#[derive(Debug, Clone)]
pub enum ClientWSCommand {
    Log(Option<String>),
    Response(Option<String>),
    Discord(Option<String>),
}

impl ClientWSCommand {
    pub fn new(string: &str) -> Result<Self, ServerError> {
        if let Some((cmd_str, data)) = string.split_once(" ") {
            let cmd = CLIENT_WS_COMMAND_STRINGS.get(cmd_str).ok_or(anyhow!("Invalid command."))?;
            match cmd {
                ClientWSCommand::Log(None)      => Ok(ClientWSCommand::Log(Some(data.to_string()))),
                ClientWSCommand::Response(None) => Ok(ClientWSCommand::Response(Some(data.to_string()))),
                ClientWSCommand::Discord(None)  => Ok(ClientWSCommand::Discord(Some(data.to_string()))),
                _ => Err(anyhow!("WTF").into()) // WTF as in how did we end up here! unreachable
            }
        } else {
            let cmd = CLIENT_WS_COMMAND_STRINGS.get(string).ok_or(anyhow!("Invalid command."))?;
            Ok(cmd.clone())
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            ClientWSCommand::Log(_)      => "Log",
            ClientWSCommand::Response(_) => "Response",
            ClientWSCommand::Discord(_)  => "Discord",
            _ => "None"
        }
    }
}

pub fn router(state: Extension<Arc<AppState>>) -> Router {
    Router::new()
        .route("/new", post(generate_auth_snippet))
        .route("/authenticate", post(generate_api_snippet))
        .route("/ws", get(websocket))
        .layer(Extension(state))
}

/// Check if the request has attached valid api key, before sending them to be handled by the
/// websocket handler.
async fn websocket(
    Extension(state): Extension<Arc<AppState>>,
    ws: WebSocketUpgrade,
    header_map: HeaderMap,
) -> Result<Response, ServerError> {
    let mut conn = state.pool.get()?;
    let api_key = header_map.get("X-API-Key");
    if api_key.is_none() {
        return Ok((StatusCode::BAD_REQUEST, "Missing API key in request!").into_response());
    }

    let maybe_client = Client::get_by_api(&mut conn, api_key.unwrap().to_str()?)?;
    if let Some(client) = maybe_client {
        Ok(ws.on_upgrade(|socket| websocket_handle(socket, client, state)))
    } else {
        Ok((
            StatusCode::UNAUTHORIZED,
            "No access authorized with given api key!",
        )
            .into_response())
    }
}

// handle incoming websocket connections by storing each socket in the local state
async fn websocket_handle(socket: WebSocket, client: Client, state: Arc<AppState>) {
    let (sender, receiver) = socket.split();
    let message_queue: Arc<Mutex<VecDeque<Message>>> = Arc::new(Mutex::new(VecDeque::new()));
    let client_id = *client.id();
    let thread_handle = tokio::spawn(handle_client(client_id, receiver, state.clone()));
    let active = ActiveClient {
        sender,
        client,
        message_queue,
        thread_handle
    };
    
    
    state.active_clients.lock().await.insert(client_id, active);
}

/// Tokio task for each active client to receive incoming ws messages.
async fn handle_client(id: i64, mut receiver: SplitStream<WebSocket>, state: Arc<AppState>) {
    while let Some(message) = receiver.next().await {
        if let Err(e) = message {
            eprintln!("Error receiving message for client {}: {:?}", id, e);
            break;
        }
        dbg!(&message);
        match message.unwrap() {
            Message::Text(text) => {
                let cmd = ClientWSCommand::new(&text); // TODO unwrap
                match cmd.unwrap() {
                    ClientWSCommand::Log(log_message) => {
                        let mut conn = state.pool.get().unwrap();
                        NewClientLog {
                            client_id: id,
                            log_message: log_message.unwrap_or("-- no log body sent --".to_string())
                        }
                            .insert_into(client_logs)
                            .execute(&mut conn)
                            .expect("Failed");
                    }
                    ClientWSCommand::Response(_) => {}
                    ClientWSCommand::Discord(text) => {
                        let json_body = r#"{"name":"test","content":"test message"}"#;
                        todo!()
                    }
                }
                // desired messages
            }
            Message::Binary(_) => unimplemented!("Client Driver Unsupported"),
            Message::Ping(_) => {} // todo we should reply with a pong
            Message::Pong(_) => {} // safely ignore pong messages
            Message::Close(_) => return,
        }
        // handle different message types here
        // let mut queue = message_queue.lock().await;
        // queue.push_back(msg);
    }
}

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
pub async fn generate_auth_snippet(
    Extension(state): Extension<Arc<AppState>>,
    Form(data): Form<AuthFormData>,
) -> Result<Markup, ServerError> {
    use crate::database::schema::clients;

    let mut connection = state.pool.get()?;
    let auth_key = create_api_key(AUTHORIZATION_KEY_LENGTH);
    let client = NewClient::new(&data.name, &auth_key);
    let inserted = diesel::insert_into(clients::table)
        .values(&client)
        .returning(Client::as_returning())
        .get_result(&mut connection)?;

    Ok(html! {
        p {"Successfully created new client with name: \"" (data.name) "\"!"}
        code class="text-center p-4" {
            (auth_key)
        }
    })
}

pub async fn generate_api_snippet(
    Extension(state): Extension<Arc<AppState>>,
    Json(payload): Json<Value>,
) -> Result<Response, ServerError> {
    use crate::database::schema::clients::dsl::*;

    let mut conn = state.pool.get()?;
    let auth = payload
        .get("authcode")
        .ok_or(ServerError(anyhow!("Missing authcode in request!")))?
        .to_string();
    let auth = strip_outer_quotes(&auth);
    let new_client = Client::get_by_auth(&mut conn, auth)?
        .ok_or(ServerError(anyhow!("Invalid authcode received!")))?;

    let new_key = create_api_key(KEY_LENGTH);
    let time_now = OffsetDateTime::now_utc();

    diesel::update(&new_client)
        .set((
            api_key.eq(&new_key),
            accessed_on.eq(PrimitiveDateTime::new(time_now.date(), time_now.time())),
            status.eq("Enrolled"),
        ))
        .execute(&mut conn)?;
    // store this key along with the authorization key if valid, otherwise we should respond with an
    // error

    Ok((StatusCode::OK, new_key).into_response())
}
fn strip_outer_quotes(s: &str) -> &str {
    if s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else {
        s
    }
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
