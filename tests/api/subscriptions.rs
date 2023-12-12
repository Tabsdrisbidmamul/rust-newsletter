use rstest::rstest;
use sqlx::query;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::{format_body, spawn_app};

#[rstest]
#[case("le guin", "guin@email.com")]
#[case("ursula", "ursula_le_guin@gmail.com")]
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data(#[case] name: String, #[case] email: String) {
    // Arrange
    let app = spawn_app().await;

    // Act
    let body = format_body(&name, &email);
    Mock::given(path("email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    let response = app.post_subscriptions(body).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[rstest]
#[case("le guin", "guin@email.com")]
#[case("ursula", "ursula_le_guin@gmail.com")]
#[tokio::test]
async fn subscribe_persists_the_new_subscriber(#[case] name: String, #[case] email: String) {
    // Arrange
    let app = spawn_app().await;

    // Act
    let body = format_body(&name, &email);
    Mock::given(path("email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body).await;

    let saved = query!(
        "SELECT email, name, status FROM subscriptions WHERE email = $1",
        email
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved subscription.");

    // Assert
    assert_eq!(saved.name, name);
    assert_eq!(saved.email, email);
    assert_eq!(saved.status, "pending_confirmation")
}

#[rstest]
#[case("name=le%20guin", "missing the email")]
#[case("email=ursula_le_guin%40gmail.com", "missing the name")]
#[case("", "missing both name and email")]
#[case("email=&name=", "empty name and email values")]
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing(
    #[case] invalid_body: String,
    #[case] description: String,
) {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.post_subscriptions(invalid_body).await;

    // Assert
    assert_eq!(
        400,
        response.status().as_u16(),
        "The API did not fail with 400 Bad Request when the payload was {}",
        description
    )
}

#[rstest]
#[case("email=&name=", "empty name and email values")]
#[tokio::test]
async fn subscribe_returns_a_400_when_field_is_present_but_invalid(
    #[case] invalid_body: String,
    #[case] description: String,
) {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.post_subscriptions(invalid_body).await;

    // Assert
    assert_eq!(
        400,
        response.status().as_u16(),
        "The API did not fail with 400 Bad Request when the payload was {}",
        description
    )
}

#[rstest]
#[case("le guin", "email=le_guin@email.com")]
#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data(
    #[case] name: String,
    #[case] email: String,
) {
    // Arrange
    let app = spawn_app().await;
    let body = format_body(&name, &email);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body).await;
}

#[rstest]
#[case("le guin", "email=le_guin@email.com")]
#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link(
    #[case] name: String,
    #[case] email: String,
) {
    // Arrange
    let app = spawn_app().await;
    let body = format_body(&name, &email);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // Act
    app.post_subscriptions(body).await;

    // Assert
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_links = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();

        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let html_link = get_links(&body["HtmlBody"].as_str().unwrap());
    let text_link = get_links(&body["TextBody"].as_str().unwrap());

    assert_eq!(html_link, text_link);
}
