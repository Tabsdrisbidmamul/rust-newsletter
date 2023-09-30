use crate::routes::{health_check, subscribe};
use actix_web::{dev::Server, web, App, HttpServer};
use std::net::TcpListener;

///
/// Main app entry to web server.
/// Will instantiate a new Server listening onto port 8000, this is only executed (lazy) when its awaited.
///
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health-check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    Ok(server)
}
