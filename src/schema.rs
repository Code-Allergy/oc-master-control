diesel::table! {
    clients (id) {
        id -> Int8,
        name -> Text,
        status -> Text,
        description -> Text,
        location -> Text,
        revoked -> Bool,
        auth_key -> Text,
        api_key -> Text,
        created_on -> Text,
        accessed_on -> Text
    }
}