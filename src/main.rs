use std::net::TcpListener;

use zero_to_production::{configuration::get_configuration, startup::run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = get_configuration().expect("Failed to read configuration");

    let address = format!("{}:{}", config.base_endpoint, config.application_port);

    let listener = TcpListener::bind(address).expect("Failed to bind port");

    println!("listening on {:?}", listener.local_addr().unwrap());

    run(listener)?.await
}
