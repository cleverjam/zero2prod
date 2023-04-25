use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
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
    let addr_str = format!("{}:{}", config.application.host, config.application.port);

    println!("** binding address {}", addr_str);
    let address = TcpListener::bind(addr_str)?;
    let db_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(config.database.connection_string().expose_secret())
        .expect("Failed to connect to Postgres.");

    run(address, db_pool)?.await
}
