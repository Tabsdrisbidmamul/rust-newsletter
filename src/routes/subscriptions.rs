use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::{query, PgPool};
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

///
/// Subscribe method which will take in POST'ed request body, extract user's email and name, save to db, and return a 200 back to client
///
pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();

    let request_span = tracing::info_span!(
      "Adding a new subscriber",
      %request_id,
      subscriber_email = %form.email,
      subscriber_name = %form.name
    );
    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new subscriber details in the database.");

    match query!(
        r#"
      INSERT INTO subscriptions (id, email, name, subscribed_at)
      VALUES ($1, $2, $3, $4)
      "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(db_pool.get_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => HttpResponse::InternalServerError().finish(),
    }
}
