#![allow(non_snake_case)]
use clap::Parser;
use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use std::{cell::Cell, path::PathBuf};
use tokio;

use crate::app::{App, AppProps, ViewFilter};

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
    Init(InitArgs),
    Run(RunArgs),
    DebugMessage(DebugMessageArgs),
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
pub struct DebugMessageArgs {
    message_id: String,
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
pub struct RunArgs {
    #[arg(long)]
    account_config_file: Option<PathBuf>,
    config_file: Option<PathBuf>,
    database_file: Option<PathBuf>,
    database_password: Option<String>,
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
pub struct InitArgs {
    database_file: Option<PathBuf>,
    database_password: Option<String>,
    config_file: Option<PathBuf>,
    account_file: Option<PathBuf>,
}

#[derive(clap::Args)]
#[command(author, version, about, long_about = None)]
pub struct SyncArgs {
    #[arg(long)]
    count: Option<u32>,
    start_time: Option<String>,
    end_time: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    match args {
        Args::DebugMessage(args) => database::debug_message(args),
        Args::Sync(args) => {
            sync::sync_count(args.count.unwrap_or(10), None)
                .await
                .expect("sync");
        }
        Args::Init(args) => init::init(args).await.expect("missing init"),
        Args::Run(args) => {
            let view = ViewFilter::default();
            let config = app::Config::default();
            dioxus_desktop::launch_with_props(
                App,
                AppProps {
                    view_filter: Cell::new(Some(view)),
                    config: Cell::new(Some(config)),
                },
                Config::default(),
            );
        }
    }

    println!("Hello, world!");
}
