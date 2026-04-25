use ::config::ConfigError;
use deadpool_postgres::{CreatePoolError, Pool, Runtime};
use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub listen: String,
    pub pg: deadpool_postgres::Config,

    pub assets_public_path: String,
    pub assets_public_url: String,

    pub assets_private_path: String,
    pub assets_private_url: String,
    //TODO: pass dieses Feld an
    //pub asset_allowed_extensions_pattern: Regex,
    pub asset_max_size: usize,

    #[serde(default = "default_session_cookie_name")]
    pub session_cookie_name: String,
    #[serde(default)]
    pub session_cookie_secure: bool,

    #[serde(default = "default_notification_file_output_dir")]
    pub notification_file_output_dir: String,

    #[serde(default = "default_public_api_url")]
    pub public_api_url: String,

    #[serde(default = "default_public_app_url")]
    pub public_app_url: String,

    pub bootstrap_admin_username: Option<String>,
    pub bootstrap_admin_email: Option<String>,
    pub bootstrap_admin_password: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv().ok();
        config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()?
            .try_deserialize()
    }
}

fn default_session_cookie_name() -> String {
    "einerleih_session".to_string()
}

fn default_notification_file_output_dir() -> String {
    "tmp/notifications".to_string()
}

fn default_public_api_url() -> String {
    "http://localhost:8000".to_string()
}

fn default_public_app_url() -> String {
    "http://localhost:8000".to_string()
}

pub async fn setup_database(config: &Config) -> Result<Pool, CreatePoolError> {
    config
        .pg
        .create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)
}
