#![allow(non_snake_case)]
use clap::Parser;
use dioxus::prelude::*;
use dioxus_desktop::{wry, Config, WindowBuilder};
use std::{
    cell::Cell,
    path::{Path, PathBuf},
};
use tokio;

use crate::{
    app::{App, AppProps, ViewFilter},
    config::DatabaseConfig,
};

mod app;
mod components;
mod config;
mod database;
mod init;
mod messages;
mod models;
mod schema;
mod sync;

#[derive(Parser)] // requires `derive` feature
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
enum Args {
    Sync(SyncArgs),
    Load(LoadArgs),
    Init(ConfigArgs),
    Run(RunArgs),
    DebugMessage(DebugMessageArgs),
}

#[derive(clap::Args, Clone)]
#[command(author, version, about, long_about = None)]
pub struct DebugMessageArgs {
    #[arg(short)]
    message_id: String,
    #[arg(short)]
    config_file: Option<PathBuf>,
    #[arg(short)]
    database_file: Option<PathBuf>,
    #[arg(short)]
    password: Option<String>,
}

#[derive(clap::Args, Clone)]
#[command(author, version, about, long_about = None)]
pub struct RunArgs {
    #[arg(short)]
    account_config_file: Option<PathBuf>,
    #[arg(short)]
    config_file: Option<PathBuf>,
    #[arg(short)]
    database_file: Option<PathBuf>,
    #[arg(short)]
    password: Option<String>,
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
pub struct ConfigArgs {
    #[arg(short)]
    database_file: Option<PathBuf>,
    #[arg(short)]
    password: Option<String>,
    #[arg(short)]
    config_file: Option<PathBuf>,
    #[arg(short)]
    account_file: Option<PathBuf>,
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
pub struct LoadArgs {
    #[arg(long)]
    path: PathBuf,
    #[arg(short)]
    config_file: Option<PathBuf>,
    #[arg(short)]
    database_file: Option<PathBuf>,
    #[arg(short)]
    password: Option<String>,
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
pub struct SyncArgs {
    #[arg(long)]
    count: Option<u32>,
    #[arg(short)]
    start_time: Option<String>,
    #[arg(short)]
    end_time: Option<String>,
    #[arg(short)]
    config_file: Option<PathBuf>,
    #[arg(short)]
    database_file: Option<PathBuf>,
    #[arg(short)]
    password: Option<String>,
}

#[tokio::main]
async fn main() {
    // prepare_default_icon();
    let args = Args::parse();
    match args {
        Args::DebugMessage(args) => {
            let og = args.clone();
            let database_config = if let Some(database_file) = args.database_file {
                DatabaseConfig {
                    path: database_file
                        .to_str()
                        .map(|s| s.to_string())
                        .expect("database path"),
                    password_used: args.password.is_some(),
                    password: args.password,
                }
            } else {
                config::get_database(&args.config_file)
            };
            database::debug_message(&database_config, og)
        }

        Args::Load(args) => {
            let database_config = if let Some(database_file) = args.database_file {
                DatabaseConfig {
                    path: database_file
                        .to_str()
                        .map(|s| s.to_string())
                        .expect("database path"),
                    password_used: args.password.is_some(),
                    password: args.password,
                }
            } else {
                config::get_database(&args.config_file)
            };
            sync::load_files(&database_config, &args.path, None)
                .await
                .expect("sync");
        }
        Args::Sync(args) => {
            let database_config = if let Some(database_file) = args.database_file {
                DatabaseConfig {
                    path: database_file
                        .to_str()
                        .map(|s| s.to_string())
                        .expect("database path"),
                    password_used: args.password.is_some(),
                    password: args.password,
                }
            } else {
                config::get_database(&args.config_file)
            };
            sync::sync_count(&database_config, args.count.unwrap_or(10), None)
                .await
                .expect("sync");
        }
        Args::Init(args) => init::init(args).await.expect("missing init"),
        Args::Run(args) => {
            let view = ViewFilter::default();
            let config = app::Config::default();
            let database_config = if let Some(database_file) = args.database_file {
                DatabaseConfig {
                    path: database_file
                        .to_str()
                        .map(|s| s.to_string())
                        .expect("database path"),
                    password_used: args.password.is_some(),
                    password: args.password,
                }
            } else {
                config::get_database(&args.config_file)
            };
            let account_config = config::get_accounts(
                args.config_file
                    .unwrap_or_else(|| config::default_config_path()),
            )
            .pop()
            .expect("account config");

            use wry::application::window::Icon;
            let bin: &[u8] = std::include_bytes!("icon.bin");
            let icon = Icon::from_rgba(bin.to_vec(), 200, 184).expect("icon");
            dioxus_desktop::launch_with_props(
                App,
                AppProps {
                    view_filter: Cell::new(Some(view)),
                    account_config: Cell::new(Some(account_config)),
                    database_config: Cell::new(Some(database_config)),
                },
                Config::default().with_icon(icon),
            );
        }
    }

    println!("Hello, world!");
}
/* // generate icon pulled from core lib
// "src/icon.bin"
// (200, 184)
fn prepare_default_icon() {
    use image::io::Reader as ImageReader;
    use image::ImageFormat;
    use std::fs::File;
    use std::io::Cursor;
    use std::io::Write;
    use std::path::PathBuf;
    let png: &[u8] = std::include_bytes!("../logo-sized.png");
    let mut reader = ImageReader::new(Cursor::new(png));
    reader.set_format(ImageFormat::Png);
    let icon = reader.decode().unwrap();
    let bin = PathBuf::from(file!()).parent().unwrap().join("icon.bin");
    println!("{:?}", bin);
    let mut file = File::create(bin).unwrap();
    file.write_all(icon.as_bytes()).unwrap();
    println!("({}, {})", icon.width(), icon.height())
}

*/
