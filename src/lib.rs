use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpResponse, HttpServer, Responder};

///
/// Health check endpoint, returns an empty 200 OK.
/// This endpoint is used to poll the application with services like insights/ pingdom to ensure that the application is running for monitoring.
///
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

///
/// Main app entry to web server.
/// Will instantiate a new Server listening onto port 8000, this is only executed (lazy) when its awaited.
///
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().route("/health-check", web::get().to(health_check)))
        .listen(listener)?
        .run();

    Ok(server)
}
