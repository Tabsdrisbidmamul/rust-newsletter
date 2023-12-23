use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use chrono::Utc;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest::{StatusCode, Url};
use sqlx::{query, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    helpers::error_chain_fmt,
    startup::ApplicationBaseUrl,
};

#[derive(serde::Deserialize, Debug)]
pub struct FormData {
    pub email: String,
    pub name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(NewSubscriber { email, name })
    }
}

#[derive(thiserror::Error)]
pub enum SubscriberError {
    #[error("{0}")]
    ValidationError(String),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub struct StoreTokenError(sqlx::Error);

impl ResponseError for SubscriberError {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            SubscriberError::ValidationError(_) => StatusCode::BAD_REQUEST,
            SubscriberError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Debug for SubscriberError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database failure was encountered while trying to store a subscription token."
        )
    }
}

#[tracing::instrument(
  name = "Adding a new subscriber",
  skip(form, db_pool, email_client, app_base_url),
  fields(
      subscriber_email = %form.email,
      subscriber_name = %form.name
  )
)]
///
/// Subscribe method which will take in POST'ed request body, extract user's email and name, save to db, and return a 200 back to client
///
pub async fn subscribe(
    form: web::Form<FormData>,
    db_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    app_base_url: web::Data<ApplicationBaseUrl>,
) -> Result<HttpResponse, SubscriberError> {
    let new_subscriber: NewSubscriber = form
        .0
        .try_into()
        .map_err(SubscriberError::ValidationError)?;

    let mut transaction = db_pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;

    let subscriber_id = insert_subscriber(&mut transaction, &new_subscriber)
        .await
        .context("Failed to insert new subscriber in the database")?;

    let subscription_token = generate_subscription_token();
    store_token(&mut transaction, subscriber_id, &subscription_token)
        .await
        .context("Failed to store the confirmation token for a new subscriber")?;

    transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber")?;

    send_confirmation_email(
        &email_client,
        &new_subscriber,
        &app_base_url.0,
        &subscription_token,
    )
    .await
    .context("Failed to send a confirmation email")?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, app_base_url, subscription_token)
)]
///
/// Send an email confirmation to the new subscriber
///
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: &NewSubscriber,
    app_base_url: &Url,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}subscriptions/confirm?subscription_token={}",
        app_base_url.as_str(),
        subscription_token
    );

    let plain_body = format!(
        "Welcome to our newsletter!<br />\
Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );

    let html_body = format!(
        "Welcome to our newsletter!<br />\
Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );

    email_client
        .send_email(&new_subscriber.email, "Welcome!", &plain_body, &html_body)
        .await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transaction)
)]
///
/// Runs the SQL query to insert the form data into the db, will bubble the error up to caller to handle
///
async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();

    query!(
        r#"
INSERT INTO subscriptions (id, email, name, subscribed_at, status)
VALUES ($1, $2, $3, $4, 'pending_confirmation')
"#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(transaction)
    .await?;

    Ok(subscriber_id)
}

#[tracing::instrument(
  name = "Store subscription token in the database"
  skip(transaction, subscriber_id, subscription_token)
)]
///
/// Store the subscription token against the newly inserted subscriber id
///
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), StoreTokenError> {
    query!(
        r#"
  INSERT INTO subscription_tokens (subscription_token, subscriber_id)
  VALUES ($1, $2)
      "#,
        subscription_token,
        subscriber_id
    )
    .execute(transaction)
    .await
    .map_err(|e| StoreTokenError(e))?;

    Ok(())
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
