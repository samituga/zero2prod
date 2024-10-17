use crate::bootstrap::Dependencies;
use crate::configuration::{DatabaseSettings, Settings};
use crate::email::email_client::{EmailClient, EmailService};
use crate::routes::{confirm, health_check, publish_newsletter, subscribe};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use http::Uri;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::net::TcpListener;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(
        configuration: Settings,
        dependencies: Dependencies,
    ) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);

        sqlx::migrate!("./migrations")
            .run(&connection_pool)
            .await
            .expect("Failed to migrate the database");

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");

        let email_service = EmailService::new(sender_email);

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address)?;
        let port = listener.local_addr()?.port();
        let server = run(
            listener,
            connection_pool,
            email_service,
            dependencies.email_client,
            configuration.application.base_url,
        )?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub struct ApplicationBaseUrl(pub Uri);

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_service: EmailService,
    email_client: Arc<dyn EmailClient>,
    base_url: Uri,
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_service = web::Data::new(email_service);
    let email_client: web::Data<dyn EmailClient> = web::Data::from(email_client.clone());
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/newsletters", web::post().to(publish_newsletter))
            .app_data(db_pool.clone())
            .app_data(email_service.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    // TODO Eager connection?
    PgPoolOptions::new().connect_lazy_with(configuration.connect_options())
}
