use rstest::rstest;
use uuid::Uuid;

use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn you_must_be_logged_in_to_see_the_change_password_form() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.get_change_password_form().await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_change_your_password() {
    // Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    // Act
    let response = app
        .post_change_password(&serde_json::json!({
          "current_password": Uuid::new_v4().to_string(),
          "new_password": &new_password,
          "new_password_check": &new_password,
        }))
        .await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn new_password_fields_must_match() {
    // Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let another_new_password = Uuid::new_v4().to_string();

    // Act - p1 login
    app.post_login(&serde_json::json!({
      "username": &app.test_user.username,
      "password": &app.test_user.password
    }))
    .await;

    // Act - p2 try to change password
    let response = app
        .post_change_password(&serde_json::json!({
          "current_password": &app.test_user.password,
          "new_password": &new_password,
          "new_password_check": &another_new_password,
        }))
        .await;

    assert_is_redirect_to(&response, "/admin/password");

    // Act - p3 follow the redirect link
    let html_page = app.get_change_password_html().await;

    // Assert
    assert!(html_page.contains(
        "<p><i>You entered two different new passwords - the field values must match.</i></p>"
    ));
}

#[tokio::test]
async fn current_password_must_be_valid() {
    // Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let wrong_password = Uuid::new_v4().to_string();

    // Act - p1 login
    app.post_login(&serde_json::json!({
      "username": &app.test_user.username,
      "password": &app.test_user.password
    }))
    .await;

    // Act - p2 try to change password
    let response = app
        .post_change_password(&serde_json::json!({
          "current_password": &wrong_password,
          "new_password": &new_password,
          "new_password_check": &new_password,
        }))
        .await;

    // Assert
    assert_is_redirect_to(&response, "/admin/password");

    // Act - p3 follow the redirect (acts as the refresh when submitting the form)
    let html_page = app.get_change_password_html().await;

    assert!(html_page.contains("<p><i>The current password is incorrect</i></p>"));
}

#[rstest]
#[case("Password")]
#[case("1.391cwypz")]
#[case("1.h10pxkzy")]
#[tokio::test]
async fn new_password_is_too_short(#[case] password: String) {
    // Arrange
    let app = spawn_app().await;

    // Act - p1 login
    app.post_login(&serde_json::json!({
      "username": &app.test_user.username,
      "password": &app.test_user.password
    }))
    .await;

    // Act - p2 try to change password (cases where password are less than 12 characters)
    let response = app
        .post_change_password(&serde_json::json!({
          "current_password": &app.test_user.password,
          "new_password": password,
          "new_password_check": password,
        }))
        .await;

    // Assert
    assert_is_redirect_to(&response, "/admin/password");

    // Act - p3 follow the redirect (acts as the refresh when submitting the form)
    let html_page = app.get_change_password_html().await;

    assert!(html_page.contains("<p><i>The password provided is too short, passwords must be greater than 12 characters</i></p>"));
}

#[tokio::test]
async fn changing_password_works() {
    // Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    // Act - p1 login
    let login_body = serde_json::json!({
      "username": &app.test_user.username,
      "password": &app.test_user.password
    });

    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - p2 change password
    let response = app
        .post_change_password(&serde_json::json!({
          "current_password": &app.test_user.password,
          "new_password": &new_password,
          "new_password_check": &new_password,
        }))
        .await;

    assert_is_redirect_to(&response, "/admin/password");

    // Act - p3 follow the redirect (refresh from form)
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>Your password has been changed</i></p>"));

    // Act - p4 logout
    let response = app.post_logout().await;
    assert_is_redirect_to(&response, "/login");

    // Act - p5 follow the redirect (refresh)
    let html_page = app.get_login_html().await;
    assert!(html_page.contains("<p><i>You have successfully logged out</i></p>"));

    // Act - p6 login using the new password
    let login_body = serde_json::json!({
      "username": &app.test_user.username,
      "password": &new_password
    });
    let response = app.post_login(&login_body).await;

    // Assert
    assert_is_redirect_to(&response, "/admin/dashboard");
}
