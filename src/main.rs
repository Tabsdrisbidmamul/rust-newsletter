use std::net::TcpListener;

use zero_to_production::run;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind port");
    println!("listening on {:?}", listener.local_addr().unwrap());

    run(listener)?.await
}
