// @generated automatically by Diesel CLI.

diesel::table! {
    cookie (rowid) {
        rowid -> Integer,
        udid -> Text,
        did -> Text,
        created_date -> Text,
        status -> Integer,
    }
}

diesel::table! {
    profile (rowid) {
        rowid -> Integer,
        did -> Text,
        handle -> Text,
        password -> Text,
        status -> Integer,
    }
}

diesel::table! {
    profile_session (rowid) {
        rowid -> Integer,
        access_jwt -> Text,
        refresh_jwt -> Text,
        did -> Text,
        active -> Bool,
        status -> Nullable<Text>,
    }
}

diesel::table! {
    timed_mute (rowid) {
        rowid -> Integer,
        actor -> Text,
        muted_actor -> Text,
        created_date -> BigInt,
        expiration_date -> BigInt,
        status -> Integer,
    }
}

diesel::table! {
    timed_mute_word (rowid) {
        rowid -> Integer,
        actor -> Text,
        muted_word -> Text,
        created_date -> BigInt,
        expiration_date -> BigInt,
        status -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    cookie,
    profile,
    profile_session,
    timed_mute,
    timed_mute_word,
);
