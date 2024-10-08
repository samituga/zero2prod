use aws_sdk_sesv2::Client;
use aws_smithy_mocks_experimental::{mock_client, Rule, RuleMode};
use secrecy::Secret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::{Arc, LazyLock};
use uuid::Uuid;
use zero2prod::bootstrap::Dependencies;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::email_client::SesClientProvider;
use zero2prod::startup::{get_connection_pool, Application};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

pub struct TestAppBootstrap {
    ses_client_provider: Arc<dyn SesClientProvider + Send + Sync>,
}

impl TestAppBootstrap {
    pub fn builder() -> TestAppBootstrapBuilder {
        TestAppBootstrapBuilder::new()
    }

    pub async fn spawn_app(self) -> TestApp {
        LazyLock::force(&TRACING);

        let configuration = {
            let mut configuration = get_configuration().expect("Failed to read configuration.");
            configuration.database.database_name = Uuid::new_v4().to_string();
            configuration.application.port = 0;
            configuration
        };

        create_database(&configuration.database).await;

        let dependencies = Dependencies {
            email_client_provider: self.ses_client_provider,
        };

        let application = Application::build(configuration.clone(), dependencies)
            .await
            .expect("Failed to build application.");
        let address = format!("http://127.0.0.1:{}", application.port());

        let _ = tokio::spawn(application.run_until_stopped());

        TestApp {
            address,
            db_pool: get_connection_pool(&configuration.database),
        }
    }
}

pub struct TestAppBootstrapBuilder {
    ses_client_provider: Option<Arc<dyn SesClientProvider + Send + Sync>>,
}

impl TestAppBootstrapBuilder {
    pub fn new() -> Self {
        Self {
            ses_client_provider: None,
        }
    }

    pub fn ses_client_rules(mut self, rules: &[Rule]) -> Self {
        let client = mock_client!(aws_sdk_sesv2, RuleMode::Sequential, rules);

        struct MockSesClientProvider {
            client: Client,
        }

        #[async_trait::async_trait]
        impl SesClientProvider for MockSesClientProvider {
            async fn ses_client(&self) -> Client {
                self.client.clone()
            }
        }

        self.ses_client_provider = Some(Arc::new(MockSesClientProvider { client }));
        self
    }

    pub async fn spawn_app(self) -> TestApp {
        TestAppBootstrap {
            ses_client_provider: self
                .ses_client_provider
                .expect("ses_client_provider is required"),
        }
        .spawn_app()
        .await
    }
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub async fn spawn_app() -> TestApp {
    TestAppBootstrap::builder()
        .ses_client_rules(&[])
        .spawn_app()
        .await
}

async fn create_database(config: &DatabaseSettings) {
    let maintenance_settings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: Secret::new("password".to_string()),
        ..config.clone()
    };

    let mut connection = PgConnection::connect_with(&maintenance_settings.connect_options())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");
}
