use crate::repositories::{CreateTask, TaskRepository};
use axum::{extract::Extension, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

pub async fn create_task<T: TaskRepository>(
    Json(payload): Json<CreateTask>,
    Extension(repository): Extension<Arc<T>>,
) -> impl IntoResponse {
    let task = repository.create(payload);
    (StatusCode::CREATED, Json(task))
}
