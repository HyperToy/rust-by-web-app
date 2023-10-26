use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
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
