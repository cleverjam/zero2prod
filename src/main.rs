use std::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = get_configuration().expect("Failed to read configuration.");
    let addr = format!("127.0.0.1:{}", config.application_port);
    let address = TcpListener::bind(addr)?;
    run(address)?.await
}
