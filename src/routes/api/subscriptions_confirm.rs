use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use reqwest::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::helpers::error_chain_fmt;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[derive(thiserror::Error)]
pub enum SubscribeConfirmError {
    #[error("There is no subscriber associated with the provided token")]
    UnknownToken,

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for SubscribeConfirmError {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            SubscribeConfirmError::UnknownToken => StatusCode::UNAUTHORIZED,
            SubscribeConfirmError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Debug for SubscribeConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(_parameters))]
///
/// Handler to confirm subscriber in db
///
pub async fn confirm(
    _parameters: web::Query<Parameters>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, SubscribeConfirmError> {
    let subscriber_id = get_subscriber_id_from_token(&db_pool, &_parameters.subscription_token)
        .await
        .context("Failed to get subscriber with supplied subscriber id")?
        .ok_or(SubscribeConfirmError::UnknownToken)?;

    confirm_subscriber(&db_pool, subscriber_id)
        .await
        .context("Failed to confirm subscriber")?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(pool, subscription_token))]
///
/// Retrieve the subscriber from db as an Option
///
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
      SELECT subscriber_id FROM subscription_tokens 
      WHERE subscription_token = $1
    "#,
        subscription_token
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.map(|r| r.subscriber_id))
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(pool, subscriber_id))]
///
/// Update the subscriber in the db and mark their status as 'confirmed'
///
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    UPDATE subscriptions 
    SET status = 'confirmed'
    WHERE id = $1
  "#,
        subscriber_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
