use actix_web::{dev::Server, web, App, HttpResponse, HttpServer};
use std::net::TcpListener;

#[derive(serde::Deserialize, Debug)]
struct FormData {
    email: String,
    name: String,
}

///
/// Health check endpoint, returns an empty 200 OK.
/// This endpoint is used to poll the application with services like insights/ pingdom to ensure that the application is running for monitoring.
///
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

///
/// Subscribe method which will take in POST'ed request body, extract user's email and name, save to db, and return a 200 back to client
///
async fn subscribe(form: web::Form<FormData>) -> HttpResponse {
    println!("{:?}", form);
    HttpResponse::Ok().finish()
}

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
