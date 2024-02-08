use uuid::Uuid;

use crate::helpers::{assert_is_redirect_to, spawn_app};

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
    let login_body = serde_json::json!({
      "username": &app.test_user.username,
      "password": &app.test_user.password
    });
    let response = app.post_login(&login_body).await;

    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - p2 submit newsletter
    let response = app
        .post_newsletter_form(&serde_json::json!({
          "title": &title,
          "text": &text,
          "html_text": &html_text
        }))
        .await;

    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - p3 follow the redirect (form submission)
    let html_page = app.get_newsletter_portal_html().await;

    // Assert
    assert!(html_page.contains("Newsletters were submitted successfully"));
}
