use crate::helpers::spawn_app;
use anyhow::Context;
use sqlx::Executor;
use uuid::Uuid;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com&password=password{";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // Act
    let response = app.post_subscriptions(body.into()).await;

    // Assert
    assert_eq!(200, response.status().as_u16());
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    // Arrange
    let app = spawn_app().await;
    let email = String::from("ursula_le_guin@gmail.com");
    let username = String::from("le guin");
    let password = String::from("password{");
    let user_id = Uuid::new_v4();

    // Act
    let mut transaction = app
        .db_pool
        .begin()
        .await
        .expect("Failed to acquire a Postgres connection from the pool");

    let include = sqlx::query!(
        r#"
    INSERT INTO users (id, account_name, account_password, account_email)
    VALUES ($1, $2, $3, $4)
            "#,
        user_id,
        username,
        password,
        email,
    );
    transaction
        .execute(include)
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })
        .expect("failed to include in db.");
    let _ = transaction
        .commit()
        .await
        .context("Failed to commit SQL transaction to store a new subscriber.");

    // Assert
    let saved = sqlx::query!(
        "SELECT account_email, account_name, account_password FROM users WHERE id=$1",
        user_id
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.account_email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.account_name, "le guin");
    assert_eq!(saved.account_password, "password{");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=le%20guin", "missing the email and password"),
        ("name=le%20guin&password=password", "missing the email"),
        (
            "email=ursula_le_guin%40gmail.com",
            "missing the name and password",
        ),
        (
            "email=ursula_le_guin%40gmail.com&password=password",
            "missing the name",
        ),
        ("password=password{", "missing the email and name"),
        ("", "missing password, name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = app.post_subscriptions(invalid_body.into()).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
    // Arrange
    let app = spawn_app().await;
    let test_cases = vec![
        ("name=le%20guin", "missing the email and password"),
        ("name=le%20guin&password=password", "missing the email"),
        (
            "email=ursula_le_guin%40gmail.com",
            "missing the name and password",
        ),
        (
            "email=ursula_le_guin%40gmail.com&password=password",
            "missing the name",
        ),
        ("password=password{", "missing the email and name"),
        ("", "missing password, name and email"),
    ];

    for (body, description) in test_cases {
        // Act
        let response = app.post_subscriptions(body.into()).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request when the payload was {}.",
            description
        );
    }
}

#[tokio::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com&password=password{";
    // Sabotage the database
    sqlx::query!("ALTER TABLE users DROP COLUMN account_email;",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    // Act
    let response = app.post_subscriptions(body.into()).await;

    // Assert
    assert_eq!(response.status().as_u16(), 500);
}
