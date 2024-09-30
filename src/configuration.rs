use std::sync::Arc;
use std::time::Duration;

use crate::domain::SubscriberEmail;
use crate::email_client::SesClientProvider;
use aws_config::timeout::TimeoutConfig;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_sesv2::config::Credentials;
use aws_sdk_sesv2::Client;
use dotenvy::dotenv;
use secrecy::{ExposeSecret, Secret};
use serde_aux::field_attributes::deserialize_number_from_string;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::ConnectOptions;
use tokio::sync::OnceCell;

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
    region: String,
    access_key_id: String,
    secret_access_key: Secret<String>,

    // TODO SES Client configurations
    operation_timeout_secs: u64,
    operation_attempt_timeout_secs: u64,
    read_timeout_secs: u64,
    connect_timeout_secs: u64,

    #[serde(skip)]
    ses_client_singleton: Arc<OnceCell<Client>>,
}

#[async_trait::async_trait]
impl SesClientProvider for AwsSettings {
    async fn ses_client(&self) -> Client {
        let cache = Arc::clone(&self.ses_client_singleton);
        cache
            .get_or_init(|| async {
                let timeout_config = TimeoutConfig::builder()
                    .operation_timeout(Duration::from_secs(self.operation_timeout_secs))
                    .operation_attempt_timeout(Duration::from_secs(
                        self.operation_attempt_timeout_secs,
                    ))
                    .read_timeout(Duration::from_secs(self.read_timeout_secs))
                    .connect_timeout(Duration::from_secs(self.connect_timeout_secs))
                    .build();

                let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
                    .credentials_provider(Credentials::new(
                        &self.access_key_id,
                        self.secret_access_key.expose_secret(),
                        None,
                        None,
                        "manual",
                    ))
                    .region(Region::new(self.region.clone()))
                    .timeout_config(timeout_config)
                    .load()
                    .await;

                Client::new(&config)
            })
            .await
            .clone()
    }
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

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");

    if environment != Environment::Production {
        dotenv().ok();
    }

    let environment_filename = format!("{}.toml", environment.as_str());

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

#[derive(PartialEq)]
pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. \
                Use either `local` or `production`.",
                other
            )),
        }
    }
}
