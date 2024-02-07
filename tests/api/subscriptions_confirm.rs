use rstest::rstest;
use sqlx::query;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{format_body, spawn_app};

#[tokio::test]
async fn confirmations_without_token_are_rejected_with_a_400() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.address))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 400)
}

#[rstest]
#[case("le guin", "guin@email.com")]
#[case("ursula", "ursula_le_guin@gmail.com")]
#[tokio::test]
async fn the_link_returned_by_subscribe_returns_a_200_if_called(
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

    app.post_subscriptions(body).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    // Act
    let response = reqwest::get(confirmation_links.html).await.unwrap();

    // Assert
    assert_eq!(response.status().as_u16(), 200);
}

#[rstest]
#[case("le guin", "guin@email.com")]
#[case("ursula", "ursula_le_guin@gmail.com")]
#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscription(
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

    app.post_subscriptions(body).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(&email_request);

    // Act
    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // Assert
    let saved = query!(
        "SELECT email, name, status FROM subscriptions WHERE email = $1",
        email
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved subscriptions");

    assert_eq!(saved.email, email);
    assert_eq!(saved.name, name);
    assert_eq!(saved.status, "confirmed");
}

// Not enough process to handle this test
// #[tokio::test]
// async fn confirm_subscription_fails_if_there_is_a_fatal_database_error() {
//     // Arrange
//     let app = spawn_app().await;
//     let body = "name=le%20guin&email=ursula_le_guin%40mail.com";

//     Mock::given(path("/email"))
//         .and(method("POST"))
//         .respond_with(ResponseTemplate::new(200))
//         .mount(&app.email_server)
//         .await;

//     app.post_subscriptions(body.into()).await;
//     let email_request = &app.email_server.received_requests().await.unwrap()[0];
//     let confirmation_links = app.get_confirmation_links(&email_request);

//     // sabotage the db
//     sqlx::query!("ALTER TABLE subscription_tokens DROP COLUMN subscription_token;",)
//         .execute(&app.db_pool)
//         .await
//         .unwrap();

//     // Act
//     let response = reqwest::get(confirmation_links.html)
//         .await
//         .expect("Failed to make HTTP POST");

//     // Assert
//     assert_eq!(response.status().as_u16(), 500);
// }
