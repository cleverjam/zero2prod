use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::spawn_app;

#[tokio::test]
pub async fn missing_tokens_are_rejected_with_a_400() {
    let app = spawn_app().await;

    let response =
        reqwest::get(&format!("{}/subscriptions/confirm", app.address))
            .await
            .unwrap();

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
pub async fn the_link_returned_by_subscribe_returns_a_200_if_called() {
    let app = spawn_app().await;
    let body = "name=BoatyMcBoatFace&email=test_user%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    let response = reqwest::get(confirmation_links.html).await.unwrap();
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
pub async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {
    let app = spawn_app().await;
    let body = "name=BoatyMcBoatFace&email=test_user%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;

    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    reqwest::get(confirmation_links.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions.");

    assert_eq!(saved.email, "test_user@gmail.com");
    assert_eq!(saved.name, "BoatyMcBoatFace");
    assert_eq!(saved.status, "confirmed");
}

#[tokio::test]
pub async fn unmatched_tokens_are_rejected_with_401() {
    let app = spawn_app().await;
    let token = 123;
    let response = reqwest::get(&format!(
        "{}/subscriptions/confirm?subscription_token={}",
        app.address, token
    ))
    .await
    .unwrap();

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
pub async fn confirmation_fails_if_there_is_a_fatal_db_error_retrieving_token()
{
    let app = spawn_app().await;
    let token = 123;

    sqlx::query!(
        "ALTER TABLE subscription_tokens DROP COLUMN subscription_token",
    )
    .execute(&app.db_pool)
    .await
    .unwrap();

    let response = reqwest::get(&format!(
        "{}/subscriptions/confirm?subscription_token={}",
        app.address, token
    ))
    .await
    .unwrap();

    assert_eq!(response.status().as_u16(), 500);
}
#[tokio::test]
pub async fn confirmation_fails_if_there_is_a_fatal_db_error_upgrading_subscription(
) {
    let app = spawn_app().await;

    let body = "name=BoatyMcBoatFace&email=test_user%40gmail.com";
    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);

    sqlx::query!("ALTER TABLE subscriptions DROP COLUMN status",)
        .execute(&app.db_pool)
        .await
        .unwrap();

    let response = reqwest::get(confirmation_links.html).await.unwrap();

    assert_eq!(response.status().as_u16(), 500);
}
