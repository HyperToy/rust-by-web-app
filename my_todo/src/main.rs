use axum::{
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::{
    collections::HashMap,
    env,
    sync::{Arc, RwLock},
};
use thiserror::Error;

#[derive(Debug, Error)]
enum RepositoryError {
    #[error("NotFound, id is {0}")]
    NotFound(i32),
}

pub trait TaskRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    fn create(&self, payload: CreateTask) -> Task;
    fn find(&self, id: i32) -> Option<Task>;
    fn all(&self) -> Vec<Task>;
    fn update(&self, id: i32, payload: UpdateTask) -> anyhow::Result<Task>;
    fn delete(&self, id: i32) -> anyhow::Result<()>;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Task {
    id: i32,
    text: String,
    completed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CreateTask {
    text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UpdateTask {
    text: Option<String>,
    completed: Option<bool>,
}

impl Task {
    pub fn new(id: i32, text: String) -> Self {
        Self {
            id,
            text,
            completed: false,
        }
    }
}

type TaskData = HashMap<i32, Task>;

#[derive(Debug, Clone)]
pub struct TaskRepositoryForMemory {
    store: Arc<RwLock<TaskData>>,
}

impl TaskRepositoryForMemory {
    pub fn new() -> Self {
        TaskRepositoryForMemory {
            store: Arc::default(),
        }
    }
}

impl TaskRepository for TaskRepositoryForMemory {
    fn create(&self, payload: CreateTask) -> Task {
        todo!();
    }
    fn find(&self, id: i32) -> Option<Task> {
        todo!();
    }
    fn all(&self) -> Vec<Task> {
        todo!();
    }
    fn update(&self, id: i32, payload: UpdateTask) -> anyhow::Result<Task> {
        todo!();
    }
    fn delete(&self, id: i32) -> anyhow::Result<()> {
        todo!();
    }
}

#[tokio::main]
async fn main() {
    // logging
    let log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
    env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();

    let repository = TaskRepositoryForMemory::new();
    let app = create_app(repository);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn create_app<T: TaskRepository>(repository: T) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/task", post(create_task::<T>))
        .layer(Extension(Arc::new(repository)))
}

async fn root() -> &'static str {
    "Hello, world!"
}

pub async fn create_task<T: TaskRepository>(
    Json(payload): Json<CreateTask>,
    Extension(repository): Extension<Arc<T>>,
) -> impl IntoResponse {
    let task = repository.create(payload);
    (StatusCode::CREATED, Json(task))
}

#[cfg(test)]
mod test {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn should_return_hello_world() {
        let repository = TaskRepositoryForMemory::new();
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = create_app(repository).oneshot(req).await.unwrap();

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(body, "Hello, world!");
    }
}
