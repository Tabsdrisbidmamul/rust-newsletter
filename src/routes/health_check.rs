use actix_web::HttpResponse;

///
/// Health check endpoint, returns an empty 200 OK.
/// This endpoint is used to poll the application with services like insights/ pingdom to ensure that the application is running for monitoring.
///
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
