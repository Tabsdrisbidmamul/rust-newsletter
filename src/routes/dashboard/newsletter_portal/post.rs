use actix_web::{
    web::{self, ReqData},
    HttpResponse,
};
use actix_web_flash_messages::FlashMessage;
use sqlx::PgPool;

use crate::{
    authentication::UserId,
    email_client::EmailClient,
    helpers::{e400, e500, see_other},
    idempotency::{save_response, try_processing, IdempotencyKey, NextAction},
    routes::publish_newsletter,
    session_state::TypedSession,
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
    email_client: web::Data<EmailClient>,
    session: TypedSession,
    user_id: ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let idempotency_key: IdempotencyKey =
        form.0.idempotency_key.clone().try_into().map_err(e400)?;

    let transaction = match try_processing(&pool, &idempotency_key, **user_id)
        .await
        .map_err(e500)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(saved_response) => {
            success_message().send();
            return Ok(saved_response);
        }
    };

    let result = publish_newsletter(form, pool.clone(), email_client, session)
        .await
        .map_err(e500)?;

    let response = match result.status().as_u16() {
        200 => {
            success_message().send();
            see_other("/admin/newsletters")
        }
        _ => {
            FlashMessage::error("Something went wrong").send();
            see_other("/admin/login")
        }
    };

    let response = save_response(transaction, &idempotency_key, **user_id, response)
        .await
        .map_err(e500)?;
    Ok(response)
}

fn success_message() -> FlashMessage {
    FlashMessage::info("Newsletters were submitted successfully")
}
