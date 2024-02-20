use std::time::Duration;

use uuid::Uuid;
use wiremock::{
    matchers::{method, path},
    Mock, MockBuilder, ResponseTemplate,
};

use crate::{
    helpers::{assert_is_redirect_to, spawn_app},
    newsletter::create_confirmed_subscriber,
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

    when_sending_an_email()
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

    app.dispatch_all_pending_emails().await;

    // Assert
}

#[tokio::test]
async fn concurrent_form_submissions_is_handled_gracefully() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - submit two newsletter forms concurrently
    let newsletter_request_body = serde_json::json!({
      "title": "Newsletter title",
      "text": "Newsletter body as plain text",
      "html_text": "<p>Newsletter body as HTML</p>",
      "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    let response_1 = app.post_newsletter_form(&newsletter_request_body);
    let response_2 = app.post_newsletter_form(&newsletter_request_body);
    let (response_1, response_2) = tokio::join!(response_1, response_2);

    // Assert
    assert_eq!(response_1.status(), response_2.status());
    assert_eq!(
        response_1.text().await.unwrap(),
        response_2.text().await.unwrap()
    );

    app.dispatch_all_pending_emails().await;
}

#[tokio::test]
async fn transient_errors_do_not_cause_duplicate_deliveries_on_retries() {
    // Arrange
    let app = spawn_app().await;
    let newsletter_request_body = serde_json::json!({
      "title": "Newsletter title",
      "text": "Newsletter body as plain text",
      "html_text": "<p>Newsletter body as HTML</p>",
      "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    // two subscribers instead of one
    create_confirmed_subscriber(&app).await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    // Act - p1 submit newsletter form
    // We expect the email server to response with a 200 for the first submission and 500 on the second concurrent submission
    when_sending_an_email()
        .respond_with(ResponseTemplate::new(200))
        .up_to_n_times(1)
        .expect(1)
        .mount(&app.email_server)
        .await;

    when_sending_an_email()
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = app.post_newsletter_form(&newsletter_request_body).await;
    assert_eq!(response.status().as_u16(), 303);

    // Act - p2 retry submitting the form
    // when_sending_an_email()
    //     .respond_with(ResponseTemplate::new(200))
    //     .expect(1)
    //     .named("Delivery retry")
    //     .mount(&app.email_server)
    //     .await;

    let response = app.post_newsletter_form(&newsletter_request_body).await;
    assert_eq!(response.status().as_u16(), 303);
    // Assert

    app.dispatch_all_pending_emails().await;
}

fn when_sending_an_email() -> MockBuilder {
    Mock::given(path("/email")).and(method("POST"))
}
