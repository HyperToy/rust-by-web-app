use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
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

    fn write_store_ref(&self) -> RwLockWriteGuard<TaskData> {
        self.store.write().unwrap()
    }
    fn read_store_ref(&self) -> RwLockReadGuard<TaskData> {
        self.store.read().unwrap()
    }
}

impl TaskRepository for TaskRepositoryForMemory {
    fn create(&self, payload: CreateTask) -> Task {
        let mut store = self.write_store_ref();
        let id = (store.len() + 1) as i32;
        let task = Task::new(id, payload.text.clone());
        store.insert(id, task.clone());
        task
    }
    fn find(&self, id: i32) -> Option<Task> {
        let store = self.read_store_ref();
        store.get(&id).map(|task| task.clone())
    }
    fn all(&self) -> Vec<Task> {
        let store = self.read_store_ref();
        Vec::from_iter(store.values().map(|task| task.clone()))
    }
    fn update(&self, id: i32, payload: UpdateTask) -> anyhow::Result<Task> {
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
    fn delete(&self, id: i32) -> anyhow::Result<()> {
        let mut store = self.write_store_ref();
        store.remove(&id).ok_or(RepositoryError::NotFound(id))?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn task_crud_scenario() {
        let text = "task text".to_string();
        let id = 1;
        let expected = Task::new(id, text.clone());
        let repository = TaskRepositoryForMemory::new();

        // create
        let task = repository.create(CreateTask { text });
        assert_eq!(expected, task);

        // find
        let task = repository.find(task.id).unwrap();
        assert_eq!(expected, task);

        // all
        let tasks = repository.all();
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
        let res = repository.delete(id);
        assert!(res.is_ok());
    }
}
