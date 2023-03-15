use crate::{
    config::{self, default_config_path, default_database_path},
    ConfigArgs,
};
use std::{
    fs::{self, File},
    io::{self, Write},
};

pub async fn init(args: ConfigArgs) -> io::Result<()> {
    let config_file = args.config_file.unwrap_or_else(default_config_path);

    let account_file = args
        .account_file
        .unwrap_or_else(|| config::default_account_path());

    let database_file = args
        .database_file
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(default_database_path));
    let database_password = &args.password;
    //pop file names
    let mut config_dir = config_file.clone();
    config_dir.pop();
    let mut database_dir = database_file.clone();
    database_dir.pop();
    let database_file = database_file.to_str().unwrap_or_default();
    let account_file = account_file.to_str().unwrap_or_default();

    dbg!(&config_dir, &database_dir);
    fs::create_dir_all(&config_dir)?;
    fs::create_dir_all(&database_dir)?;
    let config = format!(
        "accounts = [
    \"{account_file}\"
]
[database]
path = \"{database_file}\"
password_used = {}
",
        database_password
            .as_ref()
            .map(|_| "true")
            .unwrap_or_else(|| "false")
    );
    let mut file = File::create(config_file)?;

    file.write_all(config.as_bytes())?;
    //migrate db
    Ok(())
}
