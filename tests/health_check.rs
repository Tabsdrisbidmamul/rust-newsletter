use rstest::*;
use sqlx::{Connection, PgConnection};
use std::net::TcpListener;

use zero_to_production::configuration::get_configuration;
use zero_to_production::startup::run;

async fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind port");
    let port = listener.local_addr().unwrap().port();

    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    let server = run(listener, connection).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_coverage() {
    // Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{address}/health-check"))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length())
}
#[rstest]
#[case("name=le%20guin&email=ursula_le_guin%40gmail.com")]
#[case("name=leguin&email=ursula_le_guin%40gmail.com")]
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data(#[case] body: String) {
    // Arrange
    let address = spawn_app().await;
    let configuration = get_configuration().expect("Failed to read configuration");

    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

    let client = reqwest::Client::new();

    // Act
    let response = client
        .post(format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");
}

#[rstest]
#[case("name=le%20guin", "missing the email")]
#[case("email=ursula_le_guin%40gmail.com", "missing the name")]
#[case("", "missing both name and email")]
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing(
    #[case] invalid_body: String,
    #[case] error_message: String,
) {
    // Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .post(format!("{}/subscriptions", &address))
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
        error_message
    )
}
