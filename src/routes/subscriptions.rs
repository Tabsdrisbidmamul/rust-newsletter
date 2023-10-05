use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::{query, PgPool};
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

    tracing::info!(
        "request_id {}: Adding '{}' '{}' as a new subscriber",
        request_id,
        form.email,
        form.name
    );

    tracing::info!(
        "request_id {}: Saving new subscriber details in the database",
        request_id
    );

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
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id {}: New subscriber details have been saved",
                request_id
            );
            HttpResponse::Created().finish()
        }
        Err(e) => {
            tracing::error!("request_id {}: failed to execute query {:?}", request_id, e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
