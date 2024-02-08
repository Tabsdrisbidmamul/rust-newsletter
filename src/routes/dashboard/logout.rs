use actix_web::HttpResponse;
use actix_web_flash_messages::FlashMessage;

use crate::{helpers::see_other, session_state::TypedSession};

pub async fn logout(session: TypedSession) -> Result<HttpResponse, actix_web::Error> {
    session.logout();
    FlashMessage::info("You have successfully logged out").send();
    return Ok(see_other("/login"));
}
