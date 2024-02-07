use actix_web::HttpResponse;
use reqwest::header::LOCATION;

pub fn see_other(location: &str) -> HttpResponse {
    HttpResponse::SeeOther()
        .insert_header((LOCATION, location))
        .finish()
}
