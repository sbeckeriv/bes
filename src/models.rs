// Generated by diesel_ext

#![allow(unused)]
#![allow(clippy::all)]
use crate::schema::*;

use diesel::{Insertable, Queryable, QueryableByName};

#[derive(Queryable, Debug, Default, PartialEq, QueryableByName, Clone)]
#[diesel(table_name = messages)]
pub struct MessageLite {
    pub id: Option<i32>,
    pub message_id: String,
    pub account: String,
    pub parent_id: Option<String>,
    pub subject: Option<String>,
    pub sent_at: Option<String>,
    pub message_from: Option<String>,
    pub pinned_at: Option<String>,
    pub done_at: Option<String>,
    pub reminder_at: Option<String>,
    pub folders: Option<String>,
    pub content: Option<String>,
    pub message_to: Option<String>,
    pub message_cc: Option<String>,
    pub message_bcc: Option<String>,
    pub parent_thread_key: Option<String>,
    pub sent_date: Option<i64>,
}
impl From<MessageLite> for Message {
    fn from(value: MessageLite) -> Self {
        Message {
            id: value.id,
            message_id: value.message_id,
            account: value.account,
            parent_id: value.parent_id,
            subject: value.subject,
            sent_at: value.sent_at,
            message_from: value.message_from,
            pinned_at: value.pinned_at,
            done_at: value.done_at,
            reminder_at: value.reminder_at,
            folders: value.folders,
            content: None,
            text_format: None,
            html_format: None,
            message_to: value.message_to,
            message_cc: value.message_cc,
            message_bcc: value.message_bcc,
            parent_thread_key: value.parent_thread_key,
            sent_date: value.sent_date,
        }
    }
}
#[derive(Queryable, Debug, Default, PartialEq, Insertable, Clone)]
pub struct Message {
    pub id: Option<i32>,
    pub message_id: String,
    pub account: String,
    pub parent_id: Option<String>,
    pub subject: Option<String>,
    pub sent_at: Option<String>,
    pub message_from: Option<String>,
    pub pinned_at: Option<String>,
    pub done_at: Option<String>,
    pub reminder_at: Option<String>,
    pub folders: Option<String>,
    pub content: Option<String>,
    pub text_format: Option<String>,
    pub html_format: Option<String>,
    pub message_to: Option<String>,
    pub message_cc: Option<String>,
    pub message_bcc: Option<String>,
    pub parent_thread_key: Option<String>,
    pub sent_date: Option<i64>,
}
#[derive(Default, Insertable, PartialEq)]
#[diesel(table_name = messages)]
pub struct NewMessage {
    pub message_id: String,
    pub account: String,
    pub parent_id: Option<String>,
    pub subject: Option<String>,
    pub sent_at: Option<String>,
    pub message_from: Option<String>,
    pub pinned_at: Option<String>,
    pub done_at: Option<String>,
    pub reminder_at: Option<String>,
    pub folders: Option<String>,
    pub content: Option<String>,
    pub text_format: Option<String>,
    pub html_format: Option<String>,
    pub message_to: Option<String>,
    pub message_cc: Option<String>,
    pub message_bcc: Option<String>,
    pub parent_thread_key: Option<String>,
}

#[derive(Queryable, Debug, Default, PartialEq, Insertable)]
pub struct RawMessage {
    pub id: Option<i32>,
    pub message_id: Option<String>,
    pub message: Option<Vec<u8>>,
}

#[derive(Insertable, PartialEq)]
#[diesel(table_name = raw_messages)]
pub struct NewRawMessage {
    pub message_id: String,
    pub message: Vec<u8>,
}
