use crate::config::DatabaseConfig;
use crate::models::Message;
use crate::models::MessageLite;
use crate::models::RawMessage;
use crate::schema::*;
use crate::DebugMessageArgs;
use crate::{
    config::{get_database, AccountConfig},
    messages::email,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use himalaya_lib::Email;
use mailparse::parse_mail;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn run_migration(conn: &mut SqliteConnection) {
    conn.run_pending_migrations(MIGRATIONS).unwrap();
}

pub fn establish_connection(config: Option<(&str, &Option<String>)>) -> SqliteConnection {
    let configs = config.map(|c| (c.0.to_string(), c.1)).unwrap_or_else(|| {
        let db_config = get_database(&None);
        (db_config.path.clone(), &None)
    });

    let database_url = configs.0;
    let mut conn = SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
    run_migration(&mut conn);
    conn
}

pub fn debug_message(database_config: &DatabaseConfig, args: DebugMessageArgs) {
    let mut conn = establish_connection(Some((
        database_config.path.clone().as_str(),
        &database_config.password.clone(),
    )));
    let messages = messages::dsl::messages
        .filter(messages::message_id.eq(args.message_id.clone()))
        .load::<Message>(&mut conn)
        .expect("Error loading posts");
    dbg!(messages);

    let messages = raw_messages::dsl::raw_messages
        .filter(raw_messages::message_id.eq(args.message_id))
        .load::<RawMessage>(&mut conn)
        .expect("Error loading posts");

    let messages = messages.first().expect("no raw message found for id");
    let message = messages.message.to_owned().expect("raw");
    let parsed = parse_mail(&message).expect("raw parsed");
    dbg!(&parsed.get_headers(), &parsed.get_body(),);
}

pub fn list_threads(database_config: &DatabaseConfig) -> Result<Vec<Message>, String> {
    let mut conn = establish_connection(Some((
        database_config.path.clone().as_str(),
        &database_config.password.clone(),
    )));
    let messages = messages::dsl::messages
        .select((
            messages::id,
            messages::message_id,
            messages::account,
            messages::parent_id,
            messages::subject,
            messages::sent_at,
            messages::message_from,
            messages::pinned_at,
            messages::done_at,
            messages::reminder_at,
            messages::folders,
            messages::content,
            messages::message_to,
            messages::message_cc,
            messages::message_bcc,
            messages::parent_thread_key,
            messages::sent_date,
        ))
        .order(messages::sent_date.desc())
        .limit(100)
        .load::<MessageLite>(&mut conn)
        .expect("Error loading posts");
    Ok(messages.into_iter().map(|m| m.into()).collect())
}

// (text, html)
pub fn get_message_id_content(
    database_config: &DatabaseConfig,
    message_id: &str,
) -> Option<(String, String)> {
    let mut conn = establish_connection(Some((
        database_config.path.clone().as_str(),
        &database_config.password.clone(),
    )));
    let messages = messages::dsl::messages
        .filter(messages::message_id.eq(message_id))
        .order(messages::sent_date.desc())
        .first::<Message>(&mut conn)
        .ok();
    messages.map(|m| {
        (
            m.text_format.unwrap_or_default(),
            m.html_format.unwrap_or_default(),
        )
    })
}

pub fn save_records(
    database_config: &DatabaseConfig,
    raw: RawMessage,
    record: Message,
) -> Result<(), String> {
    let mut conn = establish_connection(Some((
        database_config.path.clone().as_str(),
        &database_config.password.clone(),
    )));
    diesel::insert_into(messages::table)
        .values(&record)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;
    diesel::insert_into(raw_messages::table)
        .values(&raw)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub fn message_to_db(message: &Email, account: &AccountConfig) -> Option<(RawMessage, Message)> {
    if let Ok(message) = message.parsed() {
        let headers = message.get_headers();
        let headers = headers
            .into_iter()
            .map(|h| (h.get_key(), h.get_value()))
            .collect::<HashMap<_, _>>();
        let parent_id = headers
            .get("In-Reply-To")
            .or_else(|| headers.get("References"))
            .cloned();

        let parts = message
            .subparts
            .iter()
            .map(|p| {
                (
                    p.headers
                        .iter()
                        .filter(|h| h.get_key() == "Content-Type")
                        .nth(0)
                        .map(|s| s.get_value())
                        .unwrap_or_default(),
                    email::parsed_mail_body(p).ok(),
                )
            })
            .filter(|f| f.1.is_some())
            .collect::<HashMap<_, _>>();
        let text_key = parts.keys().find(|k| k.starts_with("text/plain"));
        let html_key = parts.keys().find(|k| k.starts_with("text/html"));
        let mut text_format = text_key.and_then(|k| parts.get(k).cloned()).flatten();
        let mut html_format = html_key.and_then(|k| parts.get(k).cloned()).flatten();
        if html_format.is_none() && text_format.is_none() {
            let header = headers.get("Content-Type");
            let header = header.map(|s| s.clone().to_owned()).unwrap_or_default();
            if header.starts_with("text/html") {
                html_format = message.get_body().ok();
            } else if header.starts_with("text/plain") {
                text_format = message.get_body().ok();
            }
        }

        let content = text_format
            .as_ref()
            .or(html_format.as_ref())
            .map(|s| s.to_owned());

        let message_id = headers
            .get("Message-ID")
            .cloned()
            .unwrap_or_else(|| "no id found!".to_owned());

        let subject = headers.get("Subject").map(|a| a.clone());
        // take parent id and look it up in the db. if there use that thread id.
        let parent_thread_key = if subject
            .as_ref()
            .map(|s| s.to_lowercase().starts_with("re:"))
            .unwrap_or(false)
        {
            subject.as_ref().map(|s| {
                let mut hasher = Sha256::new();
                hasher.update(s.replace("re:", ""));
                let result: String = format!("{:X}", hasher.finalize());
                result
            })
        } else {
            subject.as_ref().map(|s| {
                let mut hasher = Sha256::new();
                hasher.update(s);
                let result: String = format!("{:X}", hasher.finalize());
                result
            })
        };
        let record = Message {
            account: account.name.clone(),
            subject,
            sent_at: headers.get("Date").cloned(),
            message_from: headers.get("From").cloned(),
            message_to: headers.get("To").cloned(),
            message_cc: headers.get("Cc").cloned(),
            message_bcc: headers.get("Bcc").cloned(),
            folders: Some("INBOX".into()),
            message_id: message_id.clone(),
            content,
            text_format,
            html_format,
            parent_id,
            parent_thread_key,
            sent_date: date_int(
                // clippy suggestion doesnt look as nice
                &headers.get("Date").cloned().unwrap_or_default(),
            ),
            ..Default::default()
        };

        let raw = RawMessage {
            message_id: Some(message_id),
            message: Some(message.raw_bytes.to_vec()),
            ..Default::default()
        };
        Some((raw, record))
    } else {
        None
    }
}

fn date_int(date: &str) -> Option<i64> {
    let sent: Option<DateTime<Utc>> = DateTime::parse_from_rfc2822(date).ok().map(|d| d.into());
    sent.map(|s| s.timestamp())
}
