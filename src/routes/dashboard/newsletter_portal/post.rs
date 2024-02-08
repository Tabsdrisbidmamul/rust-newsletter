use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use sqlx::PgPool;

use crate::{
    email_client::EmailClient,
    helpers::{e500, see_other},
    routes::publish_newsletter,
    session_state::TypedSession,
};

#[derive(serde::Deserialize)]
pub struct NewsletterFormData {
    pub title: String,
    pub text: String,
    pub html_text: String,
}

pub async fn send_and_submit_newsletter(
    form: web::Form<NewsletterFormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    session: TypedSession,
) -> Result<HttpResponse, actix_web::Error> {
    let result = publish_newsletter(form, pool, email_client, session)
        .await
        .map_err(e500)?;

    match result.status().as_u16() {
        200 => {
            FlashMessage::info("Newsletters were submitted successfully").send();
            return Ok(see_other("/admin/newsletters"));
        }
        _ => {
            FlashMessage::error("Something went wrong").send();
            return Ok(see_other("/admin/login"));
        }
    }
}
