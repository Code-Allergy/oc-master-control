use diesel::dsl::all;
use diesel::prelude::*;
use maud::{html, Markup, Render};
use time::{Date, PrimitiveDateTime};
use crate::schema::clients::dsl::clients;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::clients)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Client {
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

pub struct MiniClient{
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
    pub fn get_all(connection: &mut PgConnection) -> Result<Vec<Client>, anyhow::Error> {
        let all_clients = clients.select(Client::as_select()).load(connection)?;
        Ok(all_clients)
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::clients)]
pub struct NewClient {
    pub(crate) name: String,
    pub(crate) auth_key: String,
    pub(crate) status: String,
    pub(crate) revoked: bool,
}

impl NewClient {
    pub fn new(name: &str, auth_key: &str) -> NewClient {
        NewClient {
            name: name.to_string(), auth_key: auth_key.to_string(), status: "Authorized".to_string(), revoked: false,
        }
    }
}

enum Status {
    Authorized,
    Enrolled,
    Connected
}