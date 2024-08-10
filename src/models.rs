use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::clients)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct Client {
    name: String,
    status: String,
    revoked: bool,
    auth_key: String,
    api_key: String,
    created_on: String,
    accessed_on: String,
}

enum Status {
    Authorized,
    Enrolled,
    Connected
}