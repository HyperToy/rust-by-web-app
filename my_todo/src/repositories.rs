use axum::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use validator::Validate;

#[derive(Debug, Error)]
enum RepositoryError {
    #[error("NotFound, id is {0}")]
    NotFound(i32),
}

#[async_trait]
pub trait TaskRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: CreateTask) -> anyhow::Result<Task>;
    async fn find(&self, id: i32) -> anyhow::Result<Task>;
    async fn all(&self) -> anyhow::Result<Vec<Task>>;
    async fn update(&self, id: i32, payload: UpdateTask) -> anyhow::Result<Task>;
    async fn delete(&self, id: i32) -> anyhow::Result<()>;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Task {
    id: i32,
    text: String,
    completed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Validate)]
pub struct CreateTask {
    #[validate(length(min = 1, message = "Can not be empty"))]
    #[validate(length(max = 100, message = "Over text length"))]
    text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Validate)]
pub struct UpdateTask {
    #[validate(length(min = 1, message = "Can not be empty"))]
    #[validate(length(max = 100, message = "Over text length"))]
    text: Option<String>,
    completed: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct TaskRepositoryForDb {
    pool: PgPool,
}

impl TaskRepositoryForDb {
    pub fn new(pool: PgPool) -> Self {
        TaskRepositoryForDb { pool }
    }
}

#[async_trait]
impl TaskRepository for TaskRepositoryForDb {
    async fn create(&self, payload: CreateTask) -> anyhow::Result<Task> {
        todo!();
    }
    async fn find(&self, id: i32) -> anyhow::Result<Task> {
        todo!();
    }
    async fn all(&self) -> anyhow::Result<Vec<Task>> {
        todo!();
    }
    async fn update(&self, id: i32, payload: UpdateTask) -> anyhow::Result<Task> {
        todo!();
    }
    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        todo!();
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use anyhow::Context;
    use axum::async_trait;
    use std::{
        collections::HashMap,
        sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
    };

    impl Task {
        pub fn new(id: i32, text: String) -> Self {
            Self {
                id,
                text,
                completed: false,
            }
        }
    }

    impl CreateTask {
        pub fn new(text: String) -> Self {
            Self { text }
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

        fn write_store_ref(&self) -> RwLockWriteGuard<TaskData> {
            self.store.write().unwrap()
        }
        fn read_store_ref(&self) -> RwLockReadGuard<TaskData> {
            self.store.read().unwrap()
        }
    }

    #[async_trait]
    impl TaskRepository for TaskRepositoryForMemory {
        async fn create(&self, payload: CreateTask) -> anyhow::Result<Task> {
            let mut store = self.write_store_ref();
            let id = (store.len() + 1) as i32;
            let task = Task::new(id, payload.text.clone());
            store.insert(id, task.clone());
            Ok(task)
        }
        async fn find(&self, id: i32) -> anyhow::Result<Task> {
            let store = self.read_store_ref();
            let task = store
                .get(&id)
                .map(|task| task.clone())
                .ok_or(RepositoryError::NotFound(id))?;
            Ok(task)
        }
        async fn all(&self) -> anyhow::Result<Vec<Task>> {
            let store = self.read_store_ref();
            Ok(Vec::from_iter(store.values().map(|task| task.clone())))
        }
        async fn update(&self, id: i32, payload: UpdateTask) -> anyhow::Result<Task> {
            let mut store = self.write_store_ref();
            let todo = store.get(&id).context(RepositoryError::NotFound(id))?;
            let text = payload.text.unwrap_or(todo.text.clone());
            let completed = payload.completed.unwrap_or(todo.completed);
            let task = Task {
                id,
                text,
                completed,
            };
            store.insert(id, task.clone());
            Ok(task)
        }
        async fn delete(&self, id: i32) -> anyhow::Result<()> {
            let mut store = self.write_store_ref();
            store.remove(&id).ok_or(RepositoryError::NotFound(id))?;
            Ok(())
        }
    }

    #[tokio::test]
    async fn task_crud_scenario() {
        let text = "task text".to_string();
        let id = 1;
        let expected = Task::new(id, text.clone());
        let repository = TaskRepositoryForMemory::new();

        // create
        let task = repository
            .create(CreateTask { text })
            .await
            .expect("failed create task");
        assert_eq!(expected, task);

        // find
        let task = repository.find(task.id).await.unwrap();
        assert_eq!(expected, task);

        // all
        let tasks = repository.all().await.expect("failed get all task");
        assert_eq!(vec![expected], tasks);

        // update
        let text = "update task text".to_string();
        let task = repository
            .update(
                1,
                UpdateTask {
                    text: Some(text.clone()),
                    completed: Some(true),
                },
            )
            .await
            .expect("failed update task.");
        assert_eq!(
            Task {
                id,
                text,
                completed: true,
            },
            task
        );

        // delete
        let res = repository.delete(id).await;
        assert!(res.is_ok());
    }
}
