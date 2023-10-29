use axum::{
    async_trait,
    extract::{Extension, FromRequest, Path, RequestParts},
    http::StatusCode,
    response::IntoResponse,
    BoxError, Json,
};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use validator::Validate;

use crate::repositories::{CreateTask, TaskRepository, UpdateTask};

#[derive(Debug)]
pub struct ValidatedJson<T>(T);

#[async_trait]
impl<T, B> FromRequest<B> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    B: http_body::Body + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req).await.map_err(|rejection| {
            let message = format!("Json parse error: [{}]", rejection);
            (StatusCode::BAD_REQUEST, message)
        })?;
        value.validate().map_err(|rejection| {
            let message = format!("Validation error: [{}]", rejection);
            (StatusCode::BAD_REQUEST, message)
        })?;
        Ok(ValidatedJson(value))
    }
}

pub async fn create_task<T: TaskRepository>(
    ValidatedJson(payload): ValidatedJson<CreateTask>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let task = repository
        .create(payload)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;
    Ok((StatusCode::CREATED, Json(task)))
}

pub async fn find_task<T: TaskRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let task = repository.find(id).await.or(Err(StatusCode::NOT_FOUND))?;
    Ok((StatusCode::OK, Json(task)))
}

pub async fn all_tasks<T: TaskRepository>(
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let tasks = repository.all().await.unwrap();
    Ok((StatusCode::OK, Json(tasks)))
}

pub async fn update_task<T: TaskRepository>(
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateTask>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let task = repository
        .update(id, payload)
        .await
        .or(Err(StatusCode::NOT_FOUND))?;
    Ok((StatusCode::CREATED, Json(task)))
}

pub async fn delete_task<T: TaskRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>,
) -> StatusCode {
    repository
        .delete(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::NOT_FOUND)
}
