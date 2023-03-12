// @generated automatically by Diesel CLI.

diesel::table! {
    messages (id) {
        id -> Nullable<Integer>,
        message_id -> Text,
        account -> Text,
        parent_id -> Nullable<Text>,
        subject -> Nullable<Text>,
        sent_at -> Nullable<Text>,
        message_from -> Nullable<Text>,
        pinned_at -> Nullable<Text>,
        done_at -> Nullable<Text>,
        reminder_at -> Nullable<Text>,
        folders -> Nullable<Text>,
        content -> Nullable<Text>,
        text_format -> Nullable<Text>,
        html_format -> Nullable<Text>,
        message_to -> Nullable<Text>,
        message_cc -> Nullable<Text>,
        message_bcc -> Nullable<Text>,
        parent_thread_key -> Nullable<Text>,
        sent_date-> Nullable<BigInt>,
    }
}

diesel::table! {
    raw_messages (id) {
        id -> Nullable<Integer>,
        message_id -> Nullable<Text>,
        message -> Nullable<Binary>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(messages, raw_messages,);
