use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;

use zero_to_production::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero-to-prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&config.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres.");

    let address = format!("{}:{}", config.base_endpoint, config.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind port");

    println!("listening on {:?}", listener.local_addr().unwrap());
    run(listener, connection_pool)?.await
}
