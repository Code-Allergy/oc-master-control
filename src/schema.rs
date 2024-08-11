// @generated automatically by Diesel CLI.

diesel::table! {
    clients (id) {
        id -> Int4,
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
