use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, ResponseError};
use anyhow::Context;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::routes::error_chain_fmt;

#[derive(Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[derive(thiserror::Error)]
pub enum ConfirmSubscriptionError {
    #[error("Invalid token.")]
    IncorrectTokenError,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for ConfirmSubscriptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for ConfirmSubscriptionError {
    fn status_code(&self) -> StatusCode {
        match self {
            ConfirmSubscriptionError::IncorrectTokenError => {
                StatusCode::UNAUTHORIZED
            }
            ConfirmSubscriptionError::UnexpectedError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

#[tracing::instrument(
    name = "Confirm a pending subscriber",
    skip(parameters, db_pool)
)]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    db_pool: web::Data<PgPool>,
) -> Result<HttpResponse, ConfirmSubscriptionError> {
    let id =
        get_subscriber_id_from_token(&parameters.subscription_token, &db_pool)
            .await
            .context("Failed to retrieve subscription id from token")?
            .ok_or(ConfirmSubscriptionError::IncorrectTokenError)?;

    confirm_subscriber(id, &db_pool)
        .await
        .context(format!("Failed to confirm subscriber ID {}", id))?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Mark subscriber as confirmed",
    skip(subscriber_id, pool)
)]
pub async fn confirm_subscriber(
    subscriber_id: Uuid,
    pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[tracing::instrument(
name = "Getting subscriber id from a token"
skip(subscription_token, pool)
)]
pub async fn get_subscriber_id_from_token(
    subscription_token: &str,
    pool: &PgPool,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token
    )
        .fetch_optional(pool)
        .await?;

    Ok(result.map(|r| r.subscriber_id))
}
