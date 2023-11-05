use super::{label::Label, RepositoryError};
use axum::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use validator::Validate;

#[async_trait]
pub trait TaskRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: CreateTask) -> anyhow::Result<TaskWithLabelFromRow>;
    async fn find(&self, id: i32) -> anyhow::Result<TaskWithLabelFromRow>;
    async fn all(&self) -> anyhow::Result<Vec<TaskWithLabelFromRow>>;
    async fn update(&self, id: i32, payload: UpdateTask) -> anyhow::Result<TaskWithLabelFromRow>;
    async fn delete(&self, id: i32) -> anyhow::Result<()>;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, FromRow)]
pub struct TaskWithLabelFromRow {
    id: i32,
    text: String,
    completed: bool,
    // label_id: Option<i32>,
    // label_name: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct TaskEntity {
    pub id: i32,
    pub text: String,
    pub completed: bool,
    pub labels: Vec<Label>,
}

fn fold_entities(rows: Vec<TaskWithLabelFromRow>) -> Vec<TaskEntity> {
    rows.iter()
        .fold(vec![], |mut accum: Vec<TaskEntity>, current| {
            // TODO: 同一 id の Task を畳み込み
            // TODO: 同一 id の場合， Label を作成し `labels` へ push

            accum.push(TaskEntity {
                id: current.id,
                text: current.text.clone(),
                completed: current.completed,
                labels: vec![],
            });
            accum
        })
}

fn fold_entity(row: TaskWithLabelFromRow) -> TaskEntity {
    let task_entities = fold_entities(vec![row]);
    let task = task_entities.first().expect("expect 1 task");

    task.clone()
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
    async fn create(&self, payload: CreateTask) -> anyhow::Result<TaskWithLabelFromRow> {
        let task = sqlx::query_as::<_, TaskWithLabelFromRow>(
            r#"
                insert into tasks (text, completed)
                values ($1, false)
                returning *
            "#,
        )
        .bind(payload.text.clone())
        .fetch_one(&self.pool)
        .await?;
        Ok(task)
    }
    async fn find(&self, id: i32) -> anyhow::Result<TaskWithLabelFromRow> {
        let task = sqlx::query_as::<_, TaskWithLabelFromRow>(
            r#"
            select * from tasks where id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(task)
    }
    async fn all(&self) -> anyhow::Result<Vec<TaskWithLabelFromRow>> {
        let tasks = sqlx::query_as::<_, TaskWithLabelFromRow>(
            r#"
                select * from tasks
                order by id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(tasks)
    }
    async fn update(&self, id: i32, payload: UpdateTask) -> anyhow::Result<TaskWithLabelFromRow> {
        let old_task = self.find(id).await?;
        let task = sqlx::query_as::<_, TaskWithLabelFromRow>(
            r#"
                update tasks
                set text = $1, completed = $2
                where id = $3
                returning * 
            "#,
        )
        .bind(payload.text.unwrap_or(old_task.text))
        .bind(payload.completed.unwrap_or(old_task.completed))
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(task)
    }
    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        sqlx::query(
            r#"
                delete from tasks where id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;

        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "database-test")]
mod test {
    use super::*;
    use dotenv::dotenv;
    use std::env;

    #[tokio::test]
    async fn crud_scenario() {
        dotenv().ok();
        let database_url = &env::var("DATABASE_URL").expect("undefined [DATABASE_URL]");
        let pool = PgPool::connect(database_url)
            .await
            .expect(&format!("fail connect database, url is [{}]", database_url));

        let repository = TaskRepositoryForDb::new(pool.clone());
        let task_text = "[crud_scenario] text";

        // create
        let created = repository
            .create(CreateTask::new(task_text.to_string()))
            .await
            .expect("[create] returned Err");
        assert_eq!(created.text, task_text);
        assert!(!created.completed);

        // find
        let task = repository
            .find(created.id)
            .await
            .expect("[find] returned Err");
        assert_eq!(created, task);

        // all
        let tasks = repository.all().await.expect("[all] returned Err");
        let task = tasks.last().unwrap();
        assert_eq!(created, *task);

        // update
        let updated_text = "[crud_scenario] updated text";
        let task = repository
            .update(
                task.id,
                UpdateTask {
                    text: Some(updated_text.to_string()),
                    completed: Some(true),
                },
            )
            .await
            .expect("[update] returned Err");
        assert_eq!(created.id, task.id);
        assert_eq!(task.text, updated_text);

        // delete
        let _ = repository
            .delete(task.id)
            .await
            .expect("[delete] returned Err");
        let res = repository.find(created.id).await; // expect not found err
        assert!(res.is_err());

        let task_rows = sqlx::query(r#"select * from tasks where id = $1"#)
            .bind(task.id)
            .fetch_all(&pool)
            .await
            .expect("[delete] tasks fetch error");
        assert!(task_rows.len() == 0);
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

    impl TaskWithLabelFromRow {
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

    type TaskData = HashMap<i32, TaskWithLabelFromRow>;
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
        async fn create(&self, payload: CreateTask) -> anyhow::Result<TaskWithLabelFromRow> {
            let mut store = self.write_store_ref();
            let id = (store.len() + 1) as i32;
            let task = TaskWithLabelFromRow::new(id, payload.text.clone());
            store.insert(id, task.clone());
            Ok(task)
        }
        async fn find(&self, id: i32) -> anyhow::Result<TaskWithLabelFromRow> {
            let store = self.read_store_ref();
            let task = store
                .get(&id)
                .map(|task| task.clone())
                .ok_or(RepositoryError::NotFound(id))?;
            Ok(task)
        }
        async fn all(&self) -> anyhow::Result<Vec<TaskWithLabelFromRow>> {
            let store = self.read_store_ref();
            Ok(Vec::from_iter(store.values().map(|task| task.clone())))
        }
        async fn update(
            &self,
            id: i32,
            payload: UpdateTask,
        ) -> anyhow::Result<TaskWithLabelFromRow> {
            let mut store = self.write_store_ref();
            let todo = store.get(&id).context(RepositoryError::NotFound(id))?;
            let text = payload.text.unwrap_or(todo.text.clone());
            let completed = payload.completed.unwrap_or(todo.completed);
            let task = TaskWithLabelFromRow {
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
        let expected = TaskWithLabelFromRow::new(id, text.clone());
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
            TaskWithLabelFromRow {
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
