mod handlers;
mod repositories;

use axum::{
    extract::Extension,
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::{env, sync::Arc};

use crate::handlers::{all_tasks, create_task, delete_task, find_task, update_task};
use crate::repositories::{TaskRepository, TaskRepositoryForMemory};

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
        .route("/task", post(create_task::<T>).get(all_tasks::<T>))
        .route(
            "/task/:id",
            get(find_task::<T>)
                .delete(delete_task::<T>)
                .patch(update_task::<T>),
        )
        .layer(Extension(Arc::new(repository)))
}

async fn root() -> &'static str {
    "Hello, world!"
}

#[cfg(test)]
mod test {
    use std::vec;

    use super::*;
    use crate::repositories::{CreateTask, Task};
    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
        response::Response,
    };
    use tower::ServiceExt;

    fn build_task_req_with_json(path: &str, method: Method, json_body: String) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(json_body))
            .unwrap()
    }

    fn build_task_req_with_empty(path: &str, method: Method) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .body(Body::empty())
            .unwrap()
    }

    async fn res_to_task(res: Response) -> Task {
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        let task = serde_json::from_str(&body)
            .expect(&format!("cannot convert Task instance. body: {}", body));
        task
    }

    #[tokio::test]
    async fn should_return_hello_world() {
        let repository = TaskRepositoryForMemory::new();
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = create_app(repository).oneshot(req).await.unwrap();

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(body, "Hello, world!");
    }

    #[tokio::test]
    async fn should_created_task() {
        let expected = Task::new(1, "should_return_created_task".to_string());

        let repository = TaskRepositoryForMemory::new();
        let req = build_task_req_with_json(
            "/task",
            Method::POST,
            r#"{ "text" : "should_return_created_task" }"#.to_string(),
        );
        let res = create_app(repository).oneshot(req).await.unwrap();
        let task = res_to_task(res).await;
        assert_eq!(expected, task);
    }

    #[tokio::test]
    async fn should_find_task() {
        let expected = Task::new(1, "should_find_task".to_string());

        let repository = TaskRepositoryForMemory::new();
        repository.create(CreateTask::new("should_find_task".to_string()));
        let req = build_task_req_with_empty("/task/1", Method::GET);
        let res = create_app(repository).oneshot(req).await.unwrap();
        let task = res_to_task(res).await;
        assert_eq!(expected, task);
    }

    #[tokio::test]
    async fn should_get_all_tasks() {
        let expected = Task::new(1, "should_get_all_tasks".to_string());
        let repository = TaskRepositoryForMemory::new();
        repository.create(CreateTask::new("should_get_all_tasks".to_string()));
        let req = build_task_req_with_empty("/task", Method::GET);
        let res = create_app(repository).oneshot(req).await.unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        let tasks: Vec<Task> = serde_json::from_str(&body)
            .expect(&format!("cannot convert Task instance. body: {}", body));
        assert_eq!(vec![expected], tasks);
    }

    #[tokio::test]
    async fn should_update_task() {
        let expected = Task::new(1, "should_update_task".to_string());

        let repository = TaskRepositoryForMemory::new();
        repository.create(CreateTask::new("before_update_task".to_string()));
        let req = build_task_req_with_json(
            "/task/1",
            Method::PATCH,
            r#"{
                "id" : 1,
                "text" : "should_update_task",
                "completed" : false
            }"#
            .to_string(),
        );
        let res = create_app(repository).oneshot(req).await.unwrap();
        let task = res_to_task(res).await;
        assert_eq!(expected, task);
    }

    #[tokio::test]
    async fn should_delete_task() {
        let repository = TaskRepositoryForMemory::new();
        repository.create(CreateTask::new("should_delete_task".to_string()));
        let req = build_task_req_with_empty("/task/1", Method::DELETE);
        let res = create_app(repository).oneshot(req).await.unwrap();
        assert_eq!(StatusCode::NO_CONTENT, res.status());
    }
}
