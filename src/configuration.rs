use crate::domain::SubscriberEmail;
use crate::environment::ENVIRONMENT;
use dotenvy::dotenv;
use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::ConnectOptions;

#[derive(serde::Deserialize, Clone)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
    pub aws: AwsSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize, Clone)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    pub fn connect_options(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
            .database(&self.database_name)
            .log_statements(log::LevelFilter::Trace)
    }
}

#[derive(serde::Deserialize, Clone)]
pub struct AwsSettings {
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: Secret<String>,

    // TODO SES Client configurations
    pub operation_timeout_secs: u64,
    pub operation_attempt_timeout_secs: u64,
    pub read_timeout_secs: u64,
    pub connect_timeout_secs: u64,
}

#[derive(serde::Deserialize, Clone)]
pub struct EmailClientSettings {
    pub sender_email: String,
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        SubscriberEmail::parse(self.sender_email.clone())
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let configuration_directory = base_path.join("configuration");

    if !ENVIRONMENT.is_production() {
        allow_dot_env_vars();
    }

    let environment_filename = format!("{}.toml", ENVIRONMENT.as_str());

    let settings = config::Config::builder()
        .add_source(config::File::from(
            configuration_directory.join("base.toml"),
        ))
        .add_source(config::File::from(
            configuration_directory.join(environment_filename),
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?;

    settings.try_deserialize::<Settings>()
}

fn allow_dot_env_vars() {
    dotenv().ok();
}
