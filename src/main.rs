use zero2prod::bootstrap::build_dependencies;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::Application;
use zero2prod::telemetry;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let telemetry_subscriber =
        telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(telemetry_subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let dependencies = build_dependencies(&configuration).await;
    let server = Application::build(configuration, dependencies).await?;

    server.run_until_stopped().await?;
    Ok(())
}
