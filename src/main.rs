use env_logger::Env;
use sqlx::PgPool;
use std::net::TcpListener;
use zero_to_production::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let config = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    let address = format!("{}:{}", config.base_endpoint, config.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind port");

    println!("listening on {:?}", listener.local_addr().unwrap());
    run(listener, connection_pool)?.await
}
