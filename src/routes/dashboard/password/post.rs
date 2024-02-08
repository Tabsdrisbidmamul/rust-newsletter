use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

use crate::{
    authentication::{validate_credentials, AuthError, Credentials, UserId},
    helpers::{e500, see_other},
    routes::get_username,
};

#[derive(serde::Deserialize)]
pub struct ChangePasswordForm {
    current_password: Secret<String>,
    new_password: Secret<String>,
    new_password_check: Secret<String>,
}

pub async fn change_password(
    form: web::Form<ChangePasswordForm>,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();

    let new_password = form.0.new_password.expose_secret();
    let new_password_check = form.0.new_password_check.expose_secret();

    if check_if_password_match(&new_password, &new_password_check) {
        return Ok(send_flash_message_and_redirect(
            "You entered two different new passwords - the field values must match.",
            "/admin/password",
        ));
    }

    if check_if_password_length_is_greater_than_12(&new_password) {
        return Ok(send_flash_message_and_redirect(
            "The password provided is too short, passwords must be greater than 12 characters",
            "/admin/password",
        ));
    }

    let username = get_username(*user_id, &pool).await.map_err(e500)?;

    let credentials = Credentials {
        username,
        password: form.0.current_password,
    };

    if let Err(e) = validate_credentials(credentials, &pool).await {
        return match e {
            AuthError::InvalidCredentials(_) => {
                FlashMessage::error("The current password is incorrect").send();
                Ok(see_other("/admin/password"))
            }
            AuthError::UnexpectedError(_) => Err(e500(e).into()),
        };
    }

    crate::authentication::change_password(*user_id, form.0.new_password, &pool)
        .await
        .map_err(e500)?;

    FlashMessage::info("Your password has been changed").send();
    Ok(see_other("/admin/password"))
}

fn send_flash_message_and_redirect(flash_message: &str, redirect: &str) -> HttpResponse {
    FlashMessage::error(flash_message.to_string()).send();
    return see_other(redirect);
}

fn check_if_password_length_is_greater_than_12(password: &str) -> bool {
    password.len() < 12 || password.len() > 129
}

fn check_if_password_match(new_password: &str, new_password_check: &str) -> bool {
    new_password != new_password_check
}
