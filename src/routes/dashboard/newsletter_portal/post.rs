use actix_web::{
    web::{self, ReqData},
    HttpResponse,
};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::PgPool;

use crate::{
    authentication::UserId,
    helpers::{e400, e500, see_other},
    idempotency::{save_response, try_processing, IdempotencyKey, NextAction},
    routes::{enqueue_delivery_tasks, insert_newsletter_issue},
};

#[derive(serde::Deserialize)]
pub struct NewsletterFormData {
    pub title: String,
    pub text: String,
    pub html_text: String,
    pub idempotency_key: String,
}

pub async fn send_and_submit_newsletter(
    form: web::Form<NewsletterFormData>,
    pool: web::Data<PgPool>,
    user_id: ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    let NewsletterFormData {
        title,
        text,
        html_text,
        idempotency_key,
    } = form.0;

    let idempotency_key: IdempotencyKey = idempotency_key.clone().try_into().map_err(e400)?;

    let mut transaction = match try_processing(&pool, &idempotency_key, &user_id)
        .await
        .map_err(e500)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(saved_response) => {
            success_message().send();
            return Ok(saved_response);
        }
    };

    let issue_id = insert_newsletter_issue(&mut transaction, &title, &text, &html_text)
        .await
        .context("Failed to store newsletter details")
        .map_err(e500)?;

    enqueue_delivery_tasks(&mut transaction, issue_id)
        .await
        .context("Failed to enqueue delivery tasks")
        .map_err(e500)?;

    let response = see_other("/admin/newsletter");

    let response = save_response(transaction, &idempotency_key, &user_id, response)
        .await
        .map_err(e500)?;

    success_message().send();

    Ok(response)
}

fn success_message() -> FlashMessage {
    FlashMessage::info("Newsletters were submitted successfully\nEmails will go out shortly")
}
