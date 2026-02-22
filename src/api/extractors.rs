use crate::common::error::AppError;
use axum::{async_trait, extract::FromRequest, http::Request, Json};
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: serde::de::DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(
        req: Request<axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::InvalidInput(e.to_string()))?;

        value.validate().map_err(AppError::ValidationError)?;

        Ok(ValidatedJson(value))
    }
}
