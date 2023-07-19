use serde::Deserialize;
use std::env;
use std::fs;

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
    pub url: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JWT {
    pub user_secret: String,
    pub service_secret: String,
    pub user_max_age: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Security {
    pub password_salt: String,
    pub root_user_password: String,
    pub auto_update_root_user: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Connections {
    pub telegram_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database: Database,
    pub jwt: JWT,
    pub security: Security,
    pub connections: Connections,
}

pub fn load() -> Config {
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() <= 1 {
        "config.toml"
    } else {
        args.get(1).unwrap()
    };
    let file_contents = fs::read_to_string(config_path);
    if file_contents.is_err() {
        panic!("error: unable to read file with path \"{}\"", config_path);
    }

    match toml::from_str(file_contents.unwrap().as_str()) {
        Ok(loaded) => loaded,
        Err(err) => {
            panic!("error: unable to deserialize config. {}", err);
        }
    }
}
