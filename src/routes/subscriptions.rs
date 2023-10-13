use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::{query, PgPool};
use uuid::Uuid;

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

#[tracing::instrument(
  name = "Adding a new subscriber",
  skip(form, db_pool),
  fields(
      subscriber_email = %form.email,
      subscriber_name = %form.name
  )
)]
///
/// Subscribe method which will take in POST'ed request body, extract user's email and name, save to db, and return a 200 back to client
///
pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> HttpResponse {
    match insert_subscriber(db_pool.get_ref(), &form).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, db_pool)
)]
///
/// Runs the SQL query to insert the form data into the db, will bubble the error up to caller to handle
///
async fn insert_subscriber(db_pool: &PgPool, form: &FormData) -> Result<(), sqlx::Error> {
    query!(
        r#"
  INSERT INTO subscriptions (id, email, name, subscribed_at)
  VALUES ($1, $2, $3, $4)
  "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}", e);
        e
    })?;

    Ok(())
}
