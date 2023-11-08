mod handlers;
mod repositories;

use crate::handlers::{
    label::{all_labels, create_label, delete_label},
    task::{all_tasks, create_task, delete_task, find_task, update_task},
};
use crate::repositories::{
    label::{LabelRepository, LabelRepositoryForDb},
    task::{TaskRepository, TaskRepositoryForDb},
};
use axum::{
    extract::Extension,
    routing::{delete, get, post},
    Router,
};
use std::net::SocketAddr;
use std::{env, sync::Arc};

use dotenv::dotenv;
use hyper::header::CONTENT_TYPE;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer, Origin};

#[tokio::main]
async fn main() {
    // logging
    let log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
    env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let database_url = &env::var("DATABASE_URL").expect("undefined [DATABASE_URL");
    tracing::debug!("start connect database...");
    let pool = PgPool::connect(database_url)
        .await
        .expect(&format!("fail connect database, url is [{}]", database_url));

    let app = create_app(
        TaskRepositoryForDb::new(pool.clone()),
        LabelRepositoryForDb::new(pool.clone()),
    );
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn create_app<Task: TaskRepository, Label: LabelRepository>(
    task_repository: Task,
    label_repository: Label,
) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/task", post(create_task::<Task>).get(all_tasks::<Task>))
        .route(
            "/task/:id",
            get(find_task::<Task>)
                .delete(delete_task::<Task>)
                .patch(update_task::<Task>),
        )
        .route(
            "/label",
            post(create_label::<Label>).get(all_labels::<Label>),
        )
        .route("/label/:id", delete(delete_label::<Label>))
        .layer(Extension(Arc::new(task_repository)))
        .layer(Extension(Arc::new(label_repository)))
        .layer(
            CorsLayer::new()
                .allow_origin(Origin::exact("http://localhost:3001".parse().unwrap()))
                .allow_methods(Any)
                .allow_headers(vec![CONTENT_TYPE]),
        )
}

async fn root() -> &'static str {
    "Hello, World!"
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::repositories::{
        label::{test_utils::LabelRepositoryForMemory, Label},
        task::{test_utils::TaskRepositoryForMemory, CreateTask, TaskEntity},
    };
    use axum::{
        body::Body,
        http::{header, Method, Request, StatusCode},
        response::Response,
    };
    use tower::ServiceExt;

    fn build_req_with_json(path: &str, method: Method, json_body: String) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(json_body))
            .unwrap()
    }

    fn build_req_with_empty(path: &str, method: Method) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .body(Body::empty())
            .unwrap()
    }

    async fn res_to_task(res: Response) -> TaskEntity {
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        let task = serde_json::from_str(&body)
            .expect(&format!("cannot convert Task instance. body: {}", body));
        task
    }

    async fn res_to_label(res: Response) -> Label {
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        let label = serde_json::from_str(&body)
            .expect(&format!("cannot convert Label instance. body: {}", body));
        label
    }

    fn label_fixture() -> (Vec<Label>, Vec<i32>) {
        let id = 999;
        (
            vec![Label {
                id,
                name: String::from("test label"),
            }],
            vec![id],
        )
    }

    #[tokio::test]
    async fn should_created_task() {
        let (labels, _) = label_fixture();
        let expected = TaskEntity::new(1, "should_return_created_task".to_string(), labels.clone());

        let req = build_req_with_json(
            "/task",
            Method::POST,
            r#"{ "text": "should_return_created_task", "labels": [999] }"#.to_string(),
        );
        let res = create_app(
            TaskRepositoryForMemory::new(labels),
            LabelRepositoryForMemory::new(),
        )
        .oneshot(req)
        .await
        .unwrap();
        let task = res_to_task(res).await;
        assert_eq!(expected, task);
    }

    #[tokio::test]
    async fn should_find_task() {
        let (labels, label_ids) = label_fixture();
        let expected = TaskEntity::new(1, "should_find_task".to_string(), labels.clone());

        let task_repository = TaskRepositoryForMemory::new(labels.clone());
        task_repository
            .create(CreateTask::new("should_find_task".to_string(), label_ids))
            .await
            .expect("failed create task");
        let req = build_req_with_empty("/task/1", Method::GET);
        let res = create_app(task_repository, LabelRepositoryForMemory::new())
            .oneshot(req)
            .await
            .unwrap();
        let task = res_to_task(res).await;
        assert_eq!(expected, task);
    }

    #[tokio::test]
    async fn should_get_all_tasks() {
        let (labels, label_ids) = label_fixture();
        let expected = TaskEntity::new(1, "should_get_all_tasks".to_string(), labels.clone());

        let task_repository = TaskRepositoryForMemory::new(labels);
        task_repository
            .create(CreateTask::new(
                "should_get_all_tasks".to_string(),
                label_ids,
            ))
            .await
            .expect("failed create task");
        let req = build_req_with_empty("/task", Method::GET);
        let res = create_app(task_repository, LabelRepositoryForMemory::new())
            .oneshot(req)
            .await
            .unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        let tasks: Vec<TaskEntity> = serde_json::from_str(&body)
            .expect(&format!("cannot convert Task instance. body: {}", body));
        assert_eq!(vec![expected], tasks);
    }

    #[tokio::test]
    async fn should_update_task() {
        let (labels, label_ids) = label_fixture();
        let expected = TaskEntity::new(1, "should_update_task".to_string(), labels.clone());

        let task_repository = TaskRepositoryForMemory::new(labels);
        task_repository
            .create(CreateTask::new("before_update_task".to_string(), label_ids))
            .await
            .expect("failed create task");
        let req = build_req_with_json(
            "/task/1",
            Method::PATCH,
            r#"{
                "text": "should_update_task",
                "completed": false
            }"#
            .to_string(),
        );
        let res = create_app(task_repository, LabelRepositoryForMemory::new())
            .oneshot(req)
            .await
            .unwrap();
        let task = res_to_task(res).await;
        assert_eq!(expected, task);
    }

    #[tokio::test]
    async fn should_delete_task() {
        let (labels, label_ids) = label_fixture();
        let task_repository = TaskRepositoryForMemory::new(labels);
        task_repository
            .create(CreateTask::new("should_delete_task".to_string(), label_ids))
            .await
            .expect("failed create task");
        let req = build_req_with_empty("/task/1", Method::DELETE);
        let res = create_app(task_repository, LabelRepositoryForMemory::new())
            .oneshot(req)
            .await
            .unwrap();
        assert_eq!(StatusCode::NO_CONTENT, res.status());
    }

    #[tokio::test]
    async fn should_created_label() {
        let (_labels, _) = label_fixture();
        let expected = Label::new(1, "should_created_label".to_string());

        let req = build_req_with_json(
            "/label",
            Method::POST,
            r#"{ "name": "should_created_label" }"#.to_string(),
        );
        let res = create_app(
            TaskRepositoryForMemory::new(Vec::new()),
            LabelRepositoryForMemory::new(),
        )
        .oneshot(req)
        .await
        .unwrap();
        let label = res_to_label(res).await;
        assert_eq!(expected, label);
    }

    #[tokio::test]
    async fn should_all_label_readed() {
        let expected = Label::new(1, "should_all_label_readed".to_string());
        let label_repository = LabelRepositoryForMemory::new();
        let label = label_repository
            .create("should_all_label_readed".to_string())
            .await
            .expect("failed create label");

        let req = build_req_with_empty("/label", Method::GET);
        let res = create_app(TaskRepositoryForMemory::new(vec![label]), label_repository)
            .oneshot(req)
            .await
            .unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        let labels: Vec<Label> = serde_json::from_str(&body).expect(&format!(
            "cannot convert Label list instance. body: {}",
            body
        ));
        assert_eq!(vec![expected], labels);
    }

    #[tokio::test]
    async fn should_delete_label() {
        let label_repository = LabelRepositoryForMemory::new();
        let label = label_repository
            .create("should_delete_label".to_string())
            .await
            .expect("failed create label");
        let req = build_req_with_empty("/label/1", Method::DELETE);
        let res = create_app(TaskRepositoryForMemory::new(vec![label]), label_repository)
            .oneshot(req)
            .await
            .unwrap();
        assert_eq!(StatusCode::NO_CONTENT, res.status());
    }
}
