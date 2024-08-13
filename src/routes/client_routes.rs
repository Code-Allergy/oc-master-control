use std::collections::HashMap;
use std::sync::Arc;
use axum::{Extension, Form, Router};
use axum::extract::ws::Message;
use axum::routing::{get, post};
use futures_util::SinkExt;
use maud::{html, Markup};
use serde::Deserialize;
use crate::AppState;
use crate::error::ServerError;
use crate::database::models::{ActiveClient, Client, MiniClient};

pub fn router(state: Extension<Arc<AppState>>) -> Router {
    Router::new()
        .route("/clients", get(clients_page))
        .route("/clients/broadcast", post(clients_broadcast))
        .layer(Extension(state))
}

async fn ws_broadcast(receiving_clients: &mut HashMap<i64, ActiveClient>,
                      message: Message) -> Result<(), ServerError> {
    for client in receiving_clients.values_mut() {
        let message = message.clone();
        client.sender.send(message).await?;
    }
    Ok(())
}

async fn clients_page(Extension(state): Extension<Arc<AppState>>) -> Result<Markup, ServerError> {
    let mut conn = state.pool.get()?;
    let received_clients = Client::get_all(&mut conn)?;

    Ok(html! {
        form hx-post="/clients/broadcast" hx-target="#broadcast-response" {
            input type="text" placeholder="Query" name="query"
                class="input input-bordered w-full max-w-xs";
        }
        #broadcast-response {}
        @for client in received_clients {
            (MiniClient::from_client(client))
        }
    })
}

#[derive(Deserialize)]
struct BroadcastForm {
    query: String,
}

async fn clients_broadcast(Extension(state): Extension<Arc<AppState>>, Form(data): Form<BroadcastForm>) {
    let mut all_clients= state.active_clients.lock().await;
    ws_broadcast(&mut all_clients, Message::Text(data.query)).await.expect("Failed broadcast!");
}