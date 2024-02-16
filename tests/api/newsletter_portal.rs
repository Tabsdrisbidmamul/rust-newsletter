use uuid::Uuid;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::{
    helpers::{assert_is_redirect_to, spawn_app},
    newsletter::{create_confirmed_subscriber, create_unconfirmed_subscriber},
};

#[tokio::test]
async fn newsletter_portal_can_only_be_accessed_by_logged_in_users() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let login_body = serde_json::json!({
      "username": &app.test_user.username,
      "password": &app.test_user.password
    });
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    let html_page = app.get_newsletter_portal_html().await;

    // Assert
    assert!(html_page.contains("Submit"));
}

#[tokio::test]
async fn post_a_newsletter_successfully() {
    // Arrange
    let app = spawn_app().await;

    let title = Uuid::new_v4().to_string();
    let text = Uuid::new_v4().to_string();
    let html_text = Uuid::new_v4().to_string();

    // Act - p1 login
    app.test_user.login(&app).await;

    // Act - p2 submit newsletter
    let response = app
        .post_newsletter_form(&serde_json::json!({
          "title": &title,
          "text": &text,
          "html_text": &html_text,
          "idempotency_key": uuid::Uuid::new_v4().to_string()
        }))
        .await;

    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - p3 follow the redirect (form submission)
    let html_page = app.get_newsletter_portal_html().await;

    // Assert
    assert!(html_page.contains("Newsletters were submitted successfully"));
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - p1 submit newsletter form
    let newsletter_request_body = serde_json::json!({
      "title": "Newsletter title",
      "text": "Newsletter body as plain text",
      "html_text": "<p>Newsletter body as HTML</p>",
      "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    let response = app.post_newsletter_form(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - p2 follow the redirect (form refresh)
    let html_page = app.get_newsletter_portal_html().await;
    assert!(html_page.contains("<p><i>Newsletters were submitted successfully</i></p>"));

    // Act - p3 submit the form again
    let response = app.post_newsletter_form(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - p4 follow the redirect (form refresh)
    let html_page = app.get_newsletter_portal_html().await;
    assert!(html_page.contains("<p><i>Newsletters were submitted successfully</i></p>"));

    // Assert
}

#[tokio::test]
async fn concurrent_form_submissions_is_handled_gracefully() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act

    // Assert
}
