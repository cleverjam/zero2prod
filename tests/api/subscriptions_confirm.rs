use reqwest::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::spawn_app;

#[tokio::test]
pub async fn invalid_tokens_are_rejected_with_a_400() {
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
    let confirmation_links = app.get_confirmation_links(&email_request);

    let response = reqwest::get(confirmation_links.html).await.unwrap();
    assert_eq!(response.status().as_u16(), 200);
}
