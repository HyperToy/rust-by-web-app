use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use crate::repositories::{CreateTask, TaskRepository, UpdateTask};

pub async fn create_task<T: TaskRepository>(
    Json(payload): Json<CreateTask>,
    Extension(repository): Extension<Arc<T>>,
) -> impl IntoResponse {
    let task = repository.create(payload);
    (StatusCode::CREATED, Json(task))
}

pub async fn find_task<T: TaskRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let task = repository.find(id).ok_or(StatusCode::NOT_FOUND)?;
    Ok((StatusCode::OK, Json(task)))
}

pub async fn all_tasks<T: TaskRepository>(
    Extension(repository): Extension<Arc<T>>,
) -> impl IntoResponse {
    let tasks = repository.all();
    (StatusCode::OK, Json(tasks))
}

pub async fn update_task<T: TaskRepository>(
    Path(id): Path<i32>,
    Json(payload): Json<UpdateTask>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let task = repository
        .update(id, payload)
        .or(Err(StatusCode::NOT_FOUND))?;
    Ok((StatusCode::CREATED, Json(task)))
}

pub async fn delete_task<T: TaskRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>,
) -> StatusCode {
    repository
        .delete(id)
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::NOT_FOUND)
}
