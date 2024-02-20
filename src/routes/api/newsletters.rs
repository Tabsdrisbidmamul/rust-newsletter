use actix_web::http::header::HeaderMap;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use base64::Engine;
use reqwest::header::{self, HeaderValue};
use secrecy::Secret;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::authentication::Credentials;
use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::helpers::error_chain_fmt;
use crate::routes::NewsletterFormData;
use crate::session_state::TypedSession;

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse {
        match self {
            PublishError::UnexpectedError(_) => {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            }
            PublishError::AuthError(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();

                response
                    .headers_mut()
                    .insert(header::WWW_AUTHENTICATE, header_value);

                response
            }
        }
    }
}

#[tracing::instrument(
  name = "Insert newsletter issue"
  skip(transaction, title, text_content, html_content)
)]
pub async fn insert_newsletter_issue(
    transaction: &mut Transaction<'_, Postgres>,
    title: &str,
    text_content: &str,
    html_content: &str,
) -> Result<Uuid, sqlx::Error> {
    let newsletter_issue_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter_issues (
          newsletter_issue_id,
          title,
          text_content,
          html_content,
          published_at
        )
        VALUES ($1, $2, $3, $4, now())
      "#,
        newsletter_issue_id,
        title,
        text_content,
        html_content
    )
    .execute(transaction)
    .await?;

    Ok(newsletter_issue_id)
}

#[tracing::instrument(
  name = "Enqueue delivery tasks"
  skip()
)]
pub async fn enqueue_delivery_tasks(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
      INSERT INTO issue_delivery_queue (
        newsletter_issue_id,
        subscriber_email
      )
      SELECT $1, email
      FROM subscriptions
      WHERE status = 'confirmed'
    "#,
        newsletter_issue_id
    )
    .execute(transaction)
    .await?;

    Ok(())
}

// Legacy no longer used
#[tracing::instrument(
    name = "Publish newsletters to subscribers",
    skip(body, pool, email_client, session),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    body: web::Form<NewsletterFormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    session: TypedSession,
) -> Result<HttpResponse, PublishError> {
    let user_id = session
        .get_user_id()
        .map_err(|_| anyhow::anyhow!("Unauthorised user"))?;

    let user_id = user_id.ok_or(PublishError::AuthError(anyhow::anyhow!(
        "Unauthorised user"
    )))?;
    tracing::Span::current().record("user_id", &tracing::field::display(user_id));

    let subscribers = get_confirmed_subscribers(&pool).await?;

    for subscriber in subscribers {
        match subscriber {
            Ok(valid_subscriber) => {
                email_client
                    .send_email(
                        &valid_subscriber.email,
                        &body.title,
                        &body.html_text,
                        &body.text,
                    )
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to send newsletter issue to {}",
                            valid_subscriber.email
                        )
                    })?;
            }

            Err(error) => {
                tracing::warn!(
                  error.cause_chain = ?error,
                  "Skipping a confirmed subscriber.\nTheir stored contact details are invalid"
                );
            }
        }
    }

    Ok(HttpResponse::Ok().finish())
}

// Legacy no longer used
#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"
          SELECT email
          FROM subscriptions
          WHERE status = 'confirmed'
        "#
    )
    .fetch_all(pool)
    .await?;

    let confirmed_subscribers = rows
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(error) => Err(anyhow::anyhow!(error)),
        })
        .collect();

    Ok(confirmed_subscribers)
}

// Legacy basic auth
#[tracing::instrument(
    name = "Basic authorisation: extract username and password",
    skip(headers)
)]
fn basic_authorisation(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context("The 'Authorization' header was missing")?
        .to_str()
        .context("The 'Authorization' header was not a valid UTF8 string")?;

    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme was not 'Basic '")?;

    let decoded_bytes = base64::engine::general_purpose::STANDARD
        .decode(base64encoded_segment)
        .context("Failed to base64-decode 'Basic' credentials")?;

    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not valid UTF8")?;

    let mut credentials = decoded_credentials.splitn(2, ':');

    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A username must be provided in 'Basic' auth"))?
        .to_string();

    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!("A password must be provided in 'Basic' auth"))?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}