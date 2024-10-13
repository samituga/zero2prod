use aws_sdk_sesv2::operation::send_email::SendEmailInput;
use aws_smithy_mocks_experimental::{mock_client, Rule, RuleMode};
use secrecy::Secret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::{Arc, LazyLock};
use uuid::Uuid;
use zero2prod::bootstrap::Dependencies;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::email::email_client::EmailClient;
use zero2prod::startup::{get_connection_pool, Application};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

pub struct TestAppBootstrap {
    email_client: Arc<dyn EmailClient>,
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
            email_client: self.email_client,
        };

        let application = Application::build(configuration.clone(), dependencies)
            .await
            .expect("Failed to build application.");
        let application_port = application.port();
        let address = format!("http://127.0.0.1:{}", application_port);

        let _ = tokio::spawn(application.run_until_stopped());

        TestApp {
            address,
            port: application_port,
            db_pool: get_connection_pool(&configuration.database),
        }
    }
}

pub struct TestAppBootstrapBuilder {
    email_client: Option<Arc<dyn EmailClient>>,
}

impl TestAppBootstrapBuilder {
    pub fn new() -> Self {
        Self { email_client: None }
    }

    pub fn aws_email_client_rules(mut self, rules: &[Rule]) -> Self {
        let client: Arc<dyn EmailClient> =
            Arc::new(mock_client!(aws_sdk_sesv2, RuleMode::Sequential, rules));

        self.email_client = Some(client);
        self
    }

    pub async fn spawn_app(self) -> TestApp {
        TestAppBootstrap {
            email_client: self.email_client.expect("email_client is required"),
        }
        .spawn_app()
        .await
    }
}

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub db_pool: PgPool,
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
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

    pub fn extract_confirmation_links(&self, request: &SendEmailInput) -> ConfirmationLinks {
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
            // Let's make sure we don't call random APIs on the web
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };

        let body = request.content().unwrap().simple().unwrap().body().unwrap();
        let html = get_link(body.html().unwrap().data());
        let plain_text = get_link(body.text().unwrap().data());

        ConfirmationLinks { html, plain_text }
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
        .aws_email_client_rules(&[])
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
