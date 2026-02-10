// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "website_status"))]
    pub struct WebsiteStatus;
}

diesel::table! {
    check_history (id) {
        #[max_length = 255]
        id -> Varchar,
        #[max_length = 255]
        website_id -> Varchar,
        checked_at -> Timestamp,
        is_up -> Bool,
        response_time_ms -> Nullable<Int4>,
        status_code -> Nullable<Int4>,
        error_message -> Nullable<Text>,
    }
}

diesel::table! {
    region (id) {
        id -> Text,
        name -> Text,
    }
}

diesel::table! {
    user (id) {
        id -> Text,
        username -> Text,
        password -> Text,
    }
}

diesel::table! {
    website (id) {
        id -> Text,
        url -> Text,
        time_added -> Timestamp,
        user_id -> Text,
        is_up -> Nullable<Bool>,
        last_checked -> Nullable<Timestamp>,
        last_down_time -> Nullable<Timestamp>,
        response_time_ms -> Nullable<Int4>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::WebsiteStatus;

    website_tick (id) {
        id -> Text,
        response_time_ms -> Int4,
        status -> WebsiteStatus,
        region_id -> Text,
        website_id -> Text,
        createdAt -> Timestamp,
    }
}

diesel::joinable!(check_history -> website (website_id));
diesel::joinable!(website -> user (user_id));
diesel::joinable!(website_tick -> region (region_id));
diesel::joinable!(website_tick -> website (website_id));

diesel::allow_tables_to_appear_in_same_query!(check_history, region, user, website, website_tick,);
