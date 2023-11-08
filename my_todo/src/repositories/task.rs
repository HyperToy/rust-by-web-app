use axum::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use validator::Validate;

use super::{label::Label, RepositoryError};

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
    async fn create(&self, payload: CreateTask) -> anyhow::Result<TaskEntity> {
        let tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, TaskFromRow>(
            r#"
                insert into tasks (text, completed)
                values ($1, false)
                returning *;
            "#,
        )
        .bind(payload.text.clone())
        .fetch_one(&self.pool)
        .await?;

        sqlx::query(
            r#"
                insert into task_labels (task_id, label_id)
                select $1, id
                from unnest($2) as t(id);
            "#,
        )
        .bind(row.id)
        .bind(payload.labels)
        .execute(&self.pool)
        .await?;

        tx.commit().await?;

        let task = self.find(row.id).await?;
        Ok(task)
    }
    async fn find(&self, id: i32) -> anyhow::Result<TaskEntity> {
        let items = sqlx::query_as::<_, TaskWithLabelFromRow>(
            r#"
                select 
                    tasks.*, 
                    labels.id as label_id, 
                    labels.name as label_name 
                from 
                    tasks 
                    left outer join task_labels as tl
                        on tasks.id = tl.task_id
                    left outer join labels
                        on tl.label_id = labels.id
                where tasks.id = $1
            "#,
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;

        let tasks = fold_entities(items);
        let task = tasks.first().ok_or(RepositoryError::NotFound(id))?;
        Ok(task.clone())
    }
    async fn all(&self) -> anyhow::Result<Vec<TaskEntity>> {
        let tasks = sqlx::query_as::<_, TaskWithLabelFromRow>(
            r#"
                select 
                    tasks.*, 
                    labels.id as label_id, 
                    labels.name as label_name 
                from 
                    tasks 
                    left outer join task_labels as tl
                        on tasks.id = tl.task_id
                    left outer join labels
                        on tl.label_id = labels.id
                order by id desc
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(fold_entities(tasks))
    }
    async fn update(&self, id: i32, payload: UpdateTask) -> anyhow::Result<TaskEntity> {
        let tx = self.pool.begin().await?;

        let old_task = self.find(id).await?;
        sqlx::query(
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
        if let Some(labels) = payload.labels {
            // task's label update
            // 一度関連するレコードを削除
            sqlx::query(
                r#"
                    delete from task_labels where task_id = $1
                "#,
            )
            .bind(id)
            .execute(&self.pool)
            .await?;

            sqlx::query(
                r#"
                    insert into task_labels (task_id, label_id)
                    select $1, id
                    from unnest($2) as t(id);
                "#,
            )
            .bind(id)
            .bind(labels)
            .execute(&self.pool)
            .await?;
        }

        tx.commit().await?;
        let task = self.find(id).await?;

        Ok(task)
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        let tx = self.pool.begin().await?;
        // task's label delete
        sqlx::query(
            r#"
                delete from task_labels where task_id=$1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;
        // task delete
        sqlx::query(
            r#"
                delete from tasks where id=$1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
            _ => RepositoryError::Unexpected(e.to_string()),
        })?;

        tx.commit().await?;

        Ok(())
    }
}

#[async_trait]
pub trait TaskRepository: Clone + std::marker::Send + std::marker::Sync + 'static {
    async fn create(&self, payload: CreateTask) -> anyhow::Result<TaskEntity>;
    async fn find(&self, id: i32) -> anyhow::Result<TaskEntity>;
    async fn all(&self) -> anyhow::Result<Vec<TaskEntity>>;
    async fn update(&self, id: i32, payload: UpdateTask) -> anyhow::Result<TaskEntity>;
    async fn delete(&self, id: i32) -> anyhow::Result<()>;
}

#[derive(Clone, PartialEq, Eq, FromRow)]
struct TaskFromRow {
    id: i32,
    text: String,
    completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct TaskWithLabelFromRow {
    id: i32,
    text: String,
    completed: bool,
    label_id: Option<i32>,
    label_name: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct TaskEntity {
    pub id: i32,
    pub text: String,
    pub completed: bool,
    pub labels: Vec<Label>,
}

fn fold_entities(rows: Vec<TaskWithLabelFromRow>) -> Vec<TaskEntity> {
    let mut rows = rows.iter();
    let mut accum: Vec<TaskEntity> = vec![];
    'outer: while let Some(row) = rows.next() {
        let mut tasks = accum.iter_mut();
        while let Some(task) = tasks.next() {
            // id が一致 = Task に紐づくラベルが複数存在している
            if task.id == row.id {
                task.labels.push(Label {
                    id: row.label_id.unwrap(),
                    name: row.label_name.clone().unwrap(),
                });
                continue 'outer;
            }
        }

        // Task の id に一致がなかった時のみ到達， TaskEntity を作成
        let labels = if row.label_id.is_some() {
            vec![Label {
                id: row.label_id.unwrap(),
                name: row.label_name.clone().unwrap(),
            }]
        } else {
            vec![]
        };

        accum.push(TaskEntity {
            id: row.id,
            text: row.text.clone(),
            completed: row.completed,
            labels,
        });
    }
    accum
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Validate)]
pub struct CreateTask {
    #[validate(length(min = 1, message = "Can not be empty"))]
    #[validate(length(max = 100, message = "Over text length"))]
    text: String,
    labels: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Validate)]
pub struct UpdateTask {
    #[validate(length(min = 1, message = "Can not be empty"))]
    #[validate(length(max = 100, message = "Over text length"))]
    text: Option<String>,
    completed: Option<bool>,
    labels: Option<Vec<i32>>,
}

#[cfg(test)]
#[cfg(feature = "database-test")]
mod test {
    use super::*;
    use dotenv::dotenv;
    use std::env;

    #[test]
    fn fold_entities_test() {
        let label_1 = Label {
            id: 1,
            name: String::from("label 1"),
        };
        let label_2 = Label {
            id: 2,
            name: String::from("label 2"),
        };
        let rows = vec![
            TaskWithLabelFromRow {
                id: 1,
                text: String::from("task 1"),
                completed: false,
                label_id: Some(label_1.id),
                label_name: Some(label_1.name.clone()),
            },
            TaskWithLabelFromRow {
                id: 1,
                text: String::from("task 1"),
                completed: false,
                label_id: Some(label_2.id),
                label_name: Some(label_2.name.clone()),
            },
            TaskWithLabelFromRow {
                id: 2,
                text: String::from("task 2"),
                completed: false,
                label_id: Some(label_1.id),
                label_name: Some(label_1.name.clone()),
            },
        ];

        let res = fold_entities(rows);
        assert_eq!(
            res,
            vec![
                TaskEntity {
                    id: 1,
                    text: String::from("task 1"),
                    completed: false,
                    labels: vec![label_1.clone(), label_2.clone()],
                },
                TaskEntity {
                    id: 2,
                    text: String::from("task 2"),
                    completed: false,
                    labels: vec![label_1],
                },
            ]
        );
    }

    #[tokio::test]
    async fn crud_scenario() {
        dotenv().ok();
        let database_url = &env::var("DATABASE_URL").expect("undefined [DATABASE_URL]");
        let pool = PgPool::connect(database_url)
            .await
            .expect(&format!("fail connect database, url is [{}]", database_url));

        // label data prepare
        let label_name = String::from("test label");
        let optional_label = sqlx::query_as::<_, Label>(
            r#"
                select * from labels where name = $1
            "#,
        )
        .bind(label_name.clone())
        .fetch_optional(&pool)
        .await
        .expect("Failed to prepare label data.");
        let label_1 = if let Some(label) = optional_label {
            label
        } else {
            let label = sqlx::query_as::<_, Label>(
                r#"
                    insert into labels ( name )
                    values ( $1 )
                    returning *
                "#,
            )
            .bind(label_name)
            .fetch_one(&pool)
            .await
            .expect("Failed to insert label data.");
            label
        };

        let repository = TaskRepositoryForDb::new(pool.clone());
        let task_text = "[crud_scenario] text";

        // create
        let created = repository
            .create(CreateTask::new(task_text.to_string(), vec![label_1.id]))
            .await
            .expect("[create] returned Err");
        assert_eq!(created.text, task_text);
        assert!(!created.completed);
        assert_eq!(*created.labels.first().unwrap(), label_1);

        // find
        let task = repository
            .find(created.id)
            .await
            .expect("[find] returned Err");
        assert_eq!(created, task);

        // all
        let tasks = repository.all().await.expect("[all] returned Err");
        let task = tasks.first().unwrap();
        assert_eq!(created, *task);

        // update
        let updated_text = "[crud_scenario] updated text";
        let task = repository
            .update(
                task.id,
                UpdateTask {
                    text: Some(updated_text.to_string()),
                    completed: Some(true),
                    labels: Some(vec![]),
                },
            )
            .await
            .expect("[update] returned Err");
        assert_eq!(created.id, task.id);
        assert_eq!(task.text, updated_text);
        assert!(task.labels.len() == 0);

        // delete
        let _ = repository
            .delete(task.id)
            .await
            .expect("[delete] returned Err");
        let res = repository.find(created.id).await; // expect not found err
        assert!(res.is_err());

        let task_rows = sqlx::query(
            r#"
                select * from tasks where id=$1
            "#,
        )
        .bind(task.id)
        .fetch_all(&pool)
        .await
        .expect("[delete] task_labelss fetch error");
        assert!(task_rows.len() == 0);

        let rows = sqlx::query(
            r#"
                select * from task_labels where task_id=$1
            "#,
        )
        .bind(task.id)
        .fetch_all(&pool)
        .await
        .expect("[delete] task_labels fetch error");
        assert!(rows.len() == 0);
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

    impl TaskEntity {
        pub fn new(id: i32, text: String, labels: Vec<Label>) -> Self {
            Self {
                id,
                text,
                completed: false,
                labels,
            }
        }
    }

    impl CreateTask {
        pub fn new(text: String, labels: Vec<i32>) -> Self {
            Self { text, labels }
        }
    }

    type TaskData = HashMap<i32, TaskEntity>;

    #[derive(Debug, Clone)]
    pub struct TaskRepositoryForMemory {
        store: Arc<RwLock<TaskData>>,
        labels: Vec<Label>,
    }

    impl TaskRepositoryForMemory {
        pub fn new(labels: Vec<Label>) -> Self {
            TaskRepositoryForMemory {
                store: Arc::default(),
                labels,
            }
        }

        fn write_store_ref(&self) -> RwLockWriteGuard<TaskData> {
            self.store.write().unwrap()
        }

        fn read_store_ref(&self) -> RwLockReadGuard<TaskData> {
            self.store.read().unwrap()
        }

        fn resolve_labels(&self, labels: Vec<i32>) -> Vec<Label> {
            let mut label_list = self.labels.iter().cloned();
            let labels = labels
                .iter()
                .map(|id| label_list.find(|label| label.id == *id).unwrap())
                .collect();
            labels
        }
    }

    #[async_trait]
    impl TaskRepository for TaskRepositoryForMemory {
        async fn create(&self, payload: CreateTask) -> anyhow::Result<TaskEntity> {
            let mut store = self.write_store_ref();
            let id = (store.len() + 1) as i32;
            let labels = self.resolve_labels(payload.labels);
            let task = TaskEntity::new(id, payload.text.clone(), labels);
            store.insert(id, task.clone());
            Ok(task)
        }

        async fn find(&self, id: i32) -> anyhow::Result<TaskEntity> {
            let store = self.read_store_ref();
            let task = store
                .get(&id)
                .map(|task| task.clone())
                .ok_or(RepositoryError::NotFound(id))?;
            Ok(task)
        }

        async fn all(&self) -> anyhow::Result<Vec<TaskEntity>> {
            let store = self.read_store_ref();
            Ok(Vec::from_iter(store.values().map(|task| task.clone())))
        }

        async fn update(&self, id: i32, payload: UpdateTask) -> anyhow::Result<TaskEntity> {
            let mut store = self.write_store_ref();
            let task = store.get(&id).context(RepositoryError::NotFound(id))?;
            let text = payload.text.unwrap_or(task.text.clone());
            let completed = payload.completed.unwrap_or(task.completed);
            let labels = match payload.labels {
                Some(label_ids) => self.resolve_labels(label_ids),
                None => task.labels.clone(),
            };
            let task = TaskEntity {
                id,
                text,
                completed,
                labels,
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

    #[cfg(test)]
    mod test {
        use super::*;

        #[tokio::test]
        async fn task_crud_scenario() {
            let text = "task text".to_string();
            let id = 1;
            let label_data = Label {
                id: 1,
                name: "test label".to_string(),
            };
            let labels = vec![label_data.clone()];
            let expected = TaskEntity {
                id,
                text: text.clone(),
                completed: false,
                labels: labels.clone(),
            };

            // create
            let label_data = Label {
                id: 1,
                name: "test label".to_string(),
            };
            let labels = vec![label_data.clone()];
            let repository = TaskRepositoryForMemory::new(labels.clone());
            let task = repository
                .create(CreateTask::new(text, vec![label_data.id]))
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
                        labels: Some(vec![]),
                    },
                )
                .await
                .expect("failed update task.");
            assert_eq!(
                TaskEntity {
                    id,
                    text,
                    completed: true,
                    labels: vec![],
                },
                task
            );

            // delete
            let res = repository.delete(id).await;
            assert!(res.is_ok());
        }
    }
}
