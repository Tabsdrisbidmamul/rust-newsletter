use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;
use uuid::Uuid;

pub async fn newsletter_form(
    flash_message: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let mut msg_html = String::new();
    for m in flash_message.iter() {
        writeln!(msg_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    let idempotency_key = Uuid::new_v4().to_string();

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"
    <!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Newsletter Portal</title>
</head>
<body>
    {msg_html}
    <form action="/admin/newsletters" method="post">
        <input type="text" name="title" placeholder="Enter a title"/>
        <textarea type="text" name="text" placeholder="Enter the text version of the newsletter"></textarea>
        <textarea type="text" name="html_text" placeholder="Enter the html version of the newsletter"></textarea>

        <input hidden type="text" name="idempotency_key" value="{idempotency_key}"/>

        <button type="submit">Submit</button>
    </form>
    <p><a href="/admin/dashboard">&lt;- Back</a></p>
</body>
</html>
    "#
        )))
}
