use chrono::{DateTime, Utc};

use crate::{
    config,
    database::{message_to_db, save_records},
    messages::{self, MessageFilter},
};

pub async fn sync_count(count: u32, account_names: Option<Vec<String>>) -> Result<u32, String> {
    let accounts = config::get_accounts(config::default_config_path());
    let accounts = if let Some(filter) = account_names {
        accounts
            .into_iter()
            .filter(|account| filter.contains(&account.name))
            .collect::<Vec<_>>()
    } else {
        accounts
    };

    for account in accounts.into_iter() {
        let messages = messages::get_messages(
            &account,
            MessageFilter {
                limit: Some(count),
                folder: "INBOX".into(),
                ..Default::default()
            },
        );

        for message in messages.to_vec().into_iter() {
            if let Some((raw, record)) = message_to_db(&message, &account) {
                save_records(raw, record).map_err(|e| format!("{:?}", e))?;
            }
        }
    }

    Ok(4)
}
