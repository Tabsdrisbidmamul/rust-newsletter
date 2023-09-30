use actix_web::{web, HttpResponse};

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    email: String,
    name: String,
}

///
/// Subscribe method which will take in POST'ed request body, extract user's email and name, save to db, and return a 200 back to client
///
pub async fn subscribe(form: web::Form<FormData>) -> HttpResponse {
    println!("{:?}", form);
    HttpResponse::Ok().finish()
}
