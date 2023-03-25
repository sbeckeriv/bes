use crate::config::DatabaseConfig;
use crate::log::{debug_log, log};
use crate::models::{Message, MessageLite, RawMessage};
use crate::schema::*;
use crate::DebugMessageArgs;
use crate::{
    config::{get_database, AccountConfig},
    messages::email,
};
use chrono::{DateTime, Utc};
use diesel::dsl::not;
use diesel::query_dsl::methods::BoxedDsl;
use diesel::sqlite::{Sqlite, SqliteConnection};
use diesel::{debug_query, prelude::*};
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

pub fn list_threads(
    database_config: &DatabaseConfig,
    filter: MessageFilter,
) -> Result<Vec<Vec<Message>>, String> {
    let mut conn = establish_connection(Some((
        database_config.path.clone().as_str(),
        &database_config.password.clone(),
    )));

    let mut query = messages::table.into_boxed();
    if let Some(q) = filter.query {
        let q = format!("%{q}%");
        query = query
            .filter(messages::content.like(q.to_owned()))
            .or_filter(messages::subject.like(q.to_owned()))
            .or_filter(messages::message_to.like(q.to_owned()))
            .or_filter(messages::message_cc.like(q.to_owned()))
            .or_filter(messages::message_from.like(q.to_owned()));
    } else {
        query = query.filter(messages::parent_thread_key.is_not_null())
    }
    let query = query
        .select(messages::parent_thread_key)
        .distinct()
        .order(messages::sent_date.desc())
        .limit(100);

    let debug = debug_query::<Sqlite, _>(&query);
    debug_log(debug);
    let thread_keys = query
        .load::<Option<String>>(&mut conn)
        .expect("Error loading threads");

    let thread_keys = thread_keys
        .into_iter()
        .map(|s| s.unwrap_or_default())
        .collect::<Vec<_>>();
    dbg!(&thread_keys);

    let query = messages::table.into_boxed();
    let query = query
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
        .filter(messages::parent_thread_key.eq_any(thread_keys))
        .order(messages::sent_date.desc());
    let debug = debug_query::<Sqlite, _>(&query);
    debug_log(debug);
    let messages = query
        .load::<MessageLite>(&mut conn)
        .expect("Error loading posts");

    let mut messages_group: HashMap<String, Vec<Message>> = HashMap::new();
    messages.into_iter().for_each(|message| {
        let message: Message = message.into();
        messages_group
            .entry(message.parent_thread_key.clone().unwrap_or_default())
            .and_modify(|vec| vec.push(message.clone()))
            .or_insert(vec![message]);
    });
    // into values is not stable but inner vec is.
    let mut results = messages_group.into_values().collect::<Vec<_>>();
    results.sort_by_cached_key(|v| date_int(&v.first().unwrap().sent_at.clone().unwrap()));
    Ok(results)
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

pub fn message_to_db(
    message: &Email,
    account: &AccountConfig,
    database_config: &DatabaseConfig,
) -> Option<(RawMessage, Message)> {
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
        let parent_thread_key = parent_thread_key(&headers, database_config);
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
            sent_date: date_int(&headers.get("Date").cloned().unwrap_or_default()),
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
#[derive(Default)]
pub struct MessageFilter {
    query: Option<String>,
    pinned: bool,
    folder: bool,
    snoozed: bool,
    archived: bool,
}
impl From<&crate::app::ViewFilter> for MessageFilter {
    fn from(value: &crate::app::ViewFilter) -> Self {
        MessageFilter {
            query: value.query.clone(),
            ..Default::default()
        }
    }
}

fn parent_thread_key(
    headers: &HashMap<String, String>,
    database_config: &DatabaseConfig,
) -> Option<String> {
    let parent_id = headers
        .get("In-Reply-To")
        .or_else(|| headers.get("References"))
        .cloned();
    if let Some(parent_id) = parent_id {
        let mut conn = establish_connection(Some((
            database_config.path.clone().as_str(),
            &database_config.password.clone(),
        )));

        match messages::dsl::messages
            .filter(messages::message_id.eq(&parent_id))
            .first::<Message>(&mut conn)
        {
            Ok(message) => {
                if message.parent_thread_key.is_some() {
                    return Some(message.parent_thread_key.unwrap().clone());
                } else {
                    log(format!(
                        "message id {} matching parent id {} does not have a parent_thread_key",
                        message.message_id, parent_id
                    ));
                }
            }
            Err(err) => log(format!(
                "Error matching parent id {}: {:#?}",
                parent_id, err
            )),
        }
    }

    headers
        .get("Subject")
        .map(|a| a.clone())
        .map(|s| {
            let s = s.to_lowercase();
            s.replace("re:", "").trim().to_owned()
        })
        .map(|s| {
            let mut hasher = Sha256::new();
            hasher.update(s);
            let result: String = format!("{:X}", hasher.finalize());
            result
        })
}
/*
use to fix up threads.
WITH RECURSIVE CTE AS (
  SELECT m.id, m.message_id, m.parent_id,CAST(m.id AS varchar) AS path
  FROM messages m
  WHERE parent_id is null
  UNION ALL
  SELECT m1.id, m1.message_id, m1.parent_id,CAST(m1.id AS varchar) || ',' || path
  FROM messages m1
  JOIN CTE ON m1.parent_id = CTE.message_id
)
SELECT *
FROM CTE
where instr(path, ',') > 0;
*/
