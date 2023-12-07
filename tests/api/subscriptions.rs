use rstest::rstest;
use sqlx::query;

use crate::helpers::spawn_app;

#[rstest]
#[case("le guin", "guin@email.com")]
#[case("ursula", "ursula_le_guin@gmail.com")]
#[tokio::test]
async fn subscribe_returns_a_201_for_valid_form_data(#[case] name: String, #[case] email: String) {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let _name = str::replace(&name, " ", "%20");
    let _email = str::replace(&email, "@", "%40");
    let body = format!("name={}&email={}", _name, _email);

    let response = client
        .post(format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    let saved = query!(
        "SELECT email, name FROM subscriptions WHERE email = $1",
        email
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved subscription.");

    // Assert
    assert_eq!(201, response.status().as_u16());
    assert_eq!(saved.name, name);
    assert_eq!(saved.email, email);
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
    let client = reqwest::Client::new();

    // Act
    let response = client
        .post(format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(invalid_body)
        .send()
        .await
        .expect("Failed to execute request.");

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
    let client = reqwest::Client::new();

    // Act
    let response = client
        .post(format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(invalid_body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(
        400,
        response.status().as_u16(),
        "The API did not fail with 400 Bad Request when the payload was {}",
        description
    )
}
