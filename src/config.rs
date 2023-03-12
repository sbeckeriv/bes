use himalaya_lib::{BackendConfig, EmailSender, ImapConfig};
use serde_derive::Deserialize;
use std::fs;
use std::{path::PathBuf, str::FromStr};
use toml::{self, Table};

pub fn default_config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from_str(".").expect("no home dir path buff issues"))
        .join(".config")
        .join("bes")
        .join("config.toml")
}

pub fn default_database_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from_str(".").expect("no home dir path buff issues"))
        .join(".config")
        .join("bes")
        .join("emails.db")
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct AccountConfig {
    pub name: String,
    pub default: bool,
    pub account: Account,
    pub imap: Imap,
}
impl AccountConfig {
    // gmail focused
    pub fn backend_config(&self) -> (himalaya_lib::AccountConfig, BackendConfig) {
        let account_config = himalaya_lib::AccountConfig {
            email: self.account.email.clone(),
            display_name: self.account.display_name.clone(),
            email_sender: EmailSender::None,
            ..Default::default()
        };

        let imap_config = ImapConfig {
            host: self.imap.host.clone(),
            port: self.imap.port as u16,
            starttls: Some(false),
            login: self.imap.login.clone(),
            passwd_cmd: self
                .imap
                .passwd_cmd
                .as_ref()
                .map(|s| s.clone())
                .or_else(|| {
                    Some(format!(
                        "echo \"{}\"",
                        self.imap
                            .passwd
                            .as_ref()
                            .map(|s| s.clone())
                            .unwrap_or_default()
                    ))
                })
                .unwrap(),
            ssl: Some(true),
            ..Default::default()
        };
        let backend_config = BackendConfig::Imap(imap_config);
        (account_config, backend_config)
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct DatabaseConfig {
    pub path: String,
    pub password: Option<String>,
    pub password_used: bool,
}
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Account {
    email: String,
    display_name: Option<String>,
    #[serde(default = "default_sender")]
    sender: String,
    name: Option<String>,
    signature_delim: Option<String>,
    signature: Option<String>,
    downloads_dir: Option<String>,
    folder_listing_page_size: Option<String>,
    folder_aliases: Option<String>,
    email_listing_page_size: Option<String>,
    email_reading_headers: Option<String>,
    email_reading_format: Option<String>,
    email_reading_verify_cmd: Option<String>,
    email_reading_decrypt_cmd: Option<String>,
    email_writing_sign_cmd: Option<String>,
    email_writing_encrypt_cmd: Option<String>,
    email_writing_headers: Option<String>,
    email_hooks: Option<String>,
    sync: Option<String>,
    sync_dir: Option<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct Imap {
    #[serde(default = "default_port")]
    port: i32,
    #[serde(default = "default_host")]
    host: String,
    starttls: Option<bool>, //Some(false),
    login: String,
    passwd_cmd: Option<String>,
    passwd: Option<String>,
    ssl: Option<bool>, // Some(true),
}
fn default_port() -> i32 {
    993
}

fn default_host() -> String {
    "imap.gmail.com".into()
}
fn default_sender() -> String {
    //EmailSender::None
    "None".into()
}

pub fn get_database(config_file: Option<PathBuf>) -> DatabaseConfig {
    let config_file = config_file.unwrap_or_else(|| default_config_path());
    let contents = match fs::read_to_string(config_file) {
        Ok(c) => c,
        Err(_) => {
            return DatabaseConfig {
                path: default_database_path()
                    .to_str()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                password: None,
                password_used: false,
            }
        }
    };

    match toml::from_str(&contents) {
        Ok(d) => d,
        Err(_) => DatabaseConfig {
            path: default_database_path()
                .to_str()
                .map(|s| s.to_string())
                .unwrap_or_default(),
            password: None,
            password_used: false,
        },
    }
}

pub fn get_accounts(config_file: PathBuf) -> Vec<AccountConfig> {
    let contents = match fs::read_to_string(config_file) {
        Ok(c) => c,
        Err(_) => {
            return vec![];
        }
    };
    let data: Option<Table> = match toml::from_str(&contents) {
        Ok(d) => Some(d),
        Err(_) => None,
    };
    let data = data.and_then(|mut d| d.remove("accounts"));
    data.map(|d| {
        d.as_array().map(|d| {
            d.iter()
                .flat_map(|s| s.as_str().map(|str| get_account_config(str)).flatten())
                .collect::<Vec<_>>()
        })
    })
    .flatten()
    .unwrap_or_default()
}

pub fn get_account_config(account_file: &str) -> Option<AccountConfig> {
    let contents = match fs::read_to_string(account_file) {
        Ok(c) => c,
        Err(_) => {
            return None;
        }
    };
    let data: Option<AccountConfig> = match toml::from_str(&contents) {
        Ok(d) => Some(d),
        Err(_) => None,
    };
    data
}
