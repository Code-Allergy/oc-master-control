use std::collections::VecDeque;
use std::sync::Arc;
use axum::extract::ws::{Message, WebSocket};
use diesel::prelude::*;
use futures_util::stream::{SplitSink};
use maud::{html, Markup, Render};
use time::{PrimitiveDateTime};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

#[derive(Queryable, Selectable, Identifiable, Clone)]
#[diesel(table_name = crate::database::schema::clients)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Client {
    id: i64,
    pub name: String,
    status: String,
    description: Option<String>,
    location: Option<String>,
    revoked: bool,
    auth_key: String,
    api_key: Option<String>,
    created_on: PrimitiveDateTime,
    accessed_on: Option<PrimitiveDateTime>,
}

/// Unused in database, but used in appstate.
pub struct ActiveClient {
    pub sender: SplitSink<WebSocket, Message>,
    pub client: Client,
    pub message_queue: Arc<Mutex<VecDeque<Message>>>,
    pub thread_handle: JoinHandle<()>
}

/// Unused in database, but used as small card size reference in rendering
pub struct MiniClient {
    pub name: String,
    status: String,
    description: Option<String>,
    location: Option<String>,
    accessed_on: Option<PrimitiveDateTime>,
}

impl MiniClient {
    pub fn from_client(original: Client) -> Self {
        MiniClient {
            name: original.name,
            status: original.status,
            description: original.description,
            location: original.location,
            accessed_on: original.accessed_on,
        }
    }
}

impl Render for MiniClient {
    fn render(&self) -> Markup {
        html! {
            .flex.border."p-2" {
                .flex.flex-col."pr-4" {
                    img src="/favicon.ico" {}
                    p { "------" }
                }
                .flex.flex-col {
                    p.font-bold { (self.name) }
                    p."flex-1" { (self.location.as_deref().unwrap_or_default()) }
                    p { (self.description.as_deref().unwrap_or_default()) }
                }
            }
        }
    }
}

impl Client {
    pub fn get_all(connection: &mut PgConnection) -> Result<Vec<Self>, anyhow::Error> {
        use crate::database::schema::clients::dsl::*;
        let all_clients = clients.select(Client::as_select()).load(connection)?;
        Ok(all_clients)
    }

    pub fn get_by_auth(
        connection: &mut PgConnection,
        auth_key_query: &str,
    ) -> Result<Option<Self>, anyhow::Error> {
        use crate::database::schema::clients::dsl::*;
        let query: Client = clients
            .filter(auth_key.eq(auth_key_query))
            .first::<Client>(connection)?;

        Ok(Some(query))
    }

    pub fn get_by_api(
        connection: &mut PgConnection,
        api_key_query: &str,
    ) -> Result<Option<Self>, anyhow::Error> {
        use crate::database::schema::clients::dsl::*;
        let query: Client = clients
            .filter(api_key.eq(api_key_query))
            .first::<Client>(connection)?;

        Ok(Some(query))
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::clients)]
pub struct NewClient {
    pub(crate) name: String,
    pub(crate) auth_key: String,
    pub(crate) status: String,
    pub(crate) revoked: bool,
}

impl NewClient {
    pub fn new(name: &str, auth: &str) -> NewClient {
        NewClient {
            name: name.to_string(),
            auth_key: auth.to_string(),
            status: "Authorized".to_string(),
            revoked: false,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::database::schema::client_logs)]
pub struct NewClientLog {
    pub(crate) client_id: i64,
    pub(crate) log_message: String,
}

enum Status {
    Authorized,
    Enrolled,
    Connected,
}
