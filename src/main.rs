use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::PgPool;

use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    /* Logging start */
    let subscriber = get_subscriber("Zero2Prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    /* Logging end */

    let config = get_configuration().expect("Failed to read configuration.");
    let addr = format!("127.0.0.1:{}", config.application_port);
    let address = TcpListener::bind(addr)?;

    let db_pool = PgPool::connect(&config.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");

    run(address, db_pool)?.await
}
