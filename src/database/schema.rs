// @generated automatically by Diesel CLI.

diesel::table! {
    client_logs (id) {
        id -> Int8,
        client_id -> Int8,
        log_time -> Timestamptz,
        log_message -> Text,
    }
}

diesel::table! {
    clients (id) {
        id -> Int8,
        name -> Text,
        status -> Text,
        description -> Nullable<Text>,
        location -> Nullable<Text>,
        revoked -> Bool,
        auth_key -> Text,
        api_key -> Nullable<Text>,
        created_on -> Timestamp,
        accessed_on -> Nullable<Timestamp>,
    }
}

diesel::joinable!(client_logs -> clients (client_id));

diesel::allow_tables_to_appear_in_same_query!(
    client_logs,
    clients,
);
