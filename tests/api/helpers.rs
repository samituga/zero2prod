use secrecy::Secret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::{Arc, LazyLock};
use uuid::Uuid;
use zero2prod::bootstrap::Dependencies;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::email::aws_email_client::SesClientFactory;
use zero2prod::email::email_client::{EmailClient, EmailClientProvider};
use zero2prod::startup::{get_connection_pool, Application};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

pub async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    let configuration = {
        let mut configuration = get_configuration().expect("Failed to read configuration.");
        configuration.database.database_name = Uuid::new_v4().to_string();
        configuration.application.port = 0;
        configuration.aws.endpoint_url = Some("localhost:9090".to_string()); // TODO mock server port
        configuration
    };

    create_database(&configuration.database).await;

    let aws_email_client: Arc<dyn EmailClient> = Arc::new(
        SesClientFactory::new(&configuration.aws)
            .email_client()
            .await,
    );

    let dependencies = Dependencies {
        email_client: aws_email_client,
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
