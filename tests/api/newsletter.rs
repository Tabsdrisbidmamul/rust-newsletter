use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{spawn_app, ConfirmationLinks, TestApp};

#[tokio::test]
async fn unauthorised_users_cannot_send_newsletters() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .form(&serde_json::json!({
          "title": "Newsletter title",
          "text": "Newsletter body as plain text",
          "html_text": "<p>Newsletter body as HTML</p>",
          "idempotency_key": uuid::Uuid::new_v4().to_string()
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_data() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let newsletter_request_body = serde_json::json!({
      "title": "Newsletter title",
    });

    let response = app
        .post_newsletters_form_string(&newsletter_request_body)
        .await;

    // Assert
    assert_eq!(
        400,
        response.status().as_u16(),
        "The API did not fail with a 400 Bad Request: {}",
        "missing content"
    )
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act - p1 login
    app.test_user.login(&app).await;

    // Act - p2 submit newsletter
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text": "Newsletter body as plain text",
        "html_text": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    let response = app
        .post_newsletters_form_string(&newsletter_request_body)
        .await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - p1 login
    app.test_user.login(&app).await;

    // Act - p2 submit newsletter
    let newsletter_request_body = serde_json::json!({
      "title": "Newsletter title",
      "text": "Newsletter body as plain text",
      "html_text": "<p>Newsletter body as HTML</p>",
      "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    let response = app
        .post_newsletters_form_string(&newsletter_request_body)
        .await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}

pub async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.get_confirmation_links(&email_request)
}

pub async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_links = create_unconfirmed_subscriber(app).await;

    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}
