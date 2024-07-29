use crate::helpers::{spawn_app, TestApp};
use anyhow::Context;
use sqlx::Executor;
use uuid::Uuid;

async fn create_subscriber(app: &TestApp) {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com&password=password{";

    // Act
    app.post_subscriptions(body.into()).await;

    // Assert
    let saved = sqlx::query!("SELECT account_email, account_name, account_password FROM users",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.account_email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.account_name, "le guin");
    assert_eq!(saved.account_password, "password{");
}

#[tokio::test]
async fn requests_missing_authorization_are_rejected() {
    // Arrange
    let app = spawn_app().await;

    let response = reqwest::Client::new()
        .post(&format!("{}/new_lair", &app.address))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "image": "Newsletter title",
            "description": "Newsletter title",
            "lon": 1.5,
            "lat": 1.5,
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

#[tokio::test]
async fn content_registered_in_db() {
    // Arrange
    let app = spawn_app().await;

    let response = reqwest::Client::new()
        .post(&format!("{}/new_lair", &app.address))
        .json(&serde_json::json!({
            "title": "Newsletter title",
            "image": "Newsletter title",
            "description": "Newsletter title",
            "lon": 1.5,
            "lat": 1.5,
        }))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    );
}

#[tokio::test]
async fn subscribe_persists_the_new_lair() {
    // Arrange
    let app = spawn_app().await;
    let title = "Newsletter title";
    let image = "Newsletter title";
    let description = "Newsletter title";
    let lon = 1.5;
    let lat = 1.5;

    let room_id = Uuid::new_v4();

    let mut transaction = app
        .db_pool
        .begin()
        .await
        .expect("Failed to acquire a Postgres connection from the pool");

    // Act
    let include = sqlx::query!(
        r#"
    INSERT INTO rooms (id, title, image, description, lon, lat, room_id)
    VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        app.test_user.user_id,
        title,
        image,
        description,
        lon,
        lat,
        room_id,
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
        "SELECT title, description, image, lon, lat FROM rooms WHERE id=$1",
        app.test_user.user_id
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved subscription.");

    assert_eq!(&saved.title, "Newsletter title");
    assert_eq!(saved.description, "Newsletter title");
    assert_eq!(saved.image, "Newsletter title");
    assert_eq!(saved.lon, 1.5);
    assert_eq!(saved.lat, 1.5);
}
