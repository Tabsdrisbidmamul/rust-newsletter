use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn redirect_to_admin_dashboard_after_login_success() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let login_body = serde_json::json!({
      "username": &app.test_user.username,
      "password": &app.test_user.password
    });
    let response = app.post_login(&login_body).await;

    // Assert
    assert_is_redirect_to(&response, "/admin/dashboard");

    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome {}", app.test_user.username)));
}

#[tokio::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.get_admin_dashboard().await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn logout_clears_session_state() {
    // Arrange
    let app = spawn_app().await;

    // Act - p1 login
    let login_body = serde_json::json!({
      "username": &app.test_user.username,
      "password": &app.test_user.password
    });
    let response = app.post_login(&login_body).await;

    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - p2 - Follow the redirect (refresh from form submission)
    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome {}", app.test_user.username)));

    // Act - p3 logout
    let response = app.post_logout().await;
    assert_is_redirect_to(&response, "/login");

    // Act p4 - follow redirect (form refresh)
    let html_page = app.get_login_html().await;
    assert!(html_page.contains(r#"<p><i>You have successfully logged out</i></p>"#));

    // Act - p5 attempt to load admin panel
    let response = app.get_admin_dashboard().await;
    assert_is_redirect_to(&response, "/login");
}
