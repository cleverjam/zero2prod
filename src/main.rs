use zero2prod::configuration::get_configuration;
use zero2prod::startup::Application;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    /* Logging start */
    let subscriber =
        get_subscriber("Zero2Prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    /* Logging end */

    let config = get_configuration().expect("Failed to read configuration.");

    let application = Application::build(config).await?;

    application.run_until_stopped().await?;
    Ok(())
}
