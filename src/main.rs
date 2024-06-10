use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;

use zero2prod::configuration::get_configuration;
use zero2prod::email_client::{AwsSesEmailSender, EmailService};
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection_pool = PgPoolOptions::new()
        .connect_with(configuration.database.with_db_name())
        .await
        .expect("Failed to connect to the database");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    let aws_client = AwsSesEmailSender::new(configuration.aws.ses_client().await);
    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");

    let email_client = EmailService::new(aws_client, sender_email);

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool, email_client)?.await
}
