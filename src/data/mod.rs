use crate::data::task::Model;
use async_trait::async_trait;
use uuid::Uuid;

pub mod flow;
pub mod task;

#[async_trait]
pub trait Repository<T> {
    async fn save(&self, new: T) -> anyhow::Result<T>;
    async fn retrieve_by_id(&self, id: Uuid) -> anyhow::Result<T>;
    async fn delete_by_id(&self, id: Uuid) -> anyhow::Result<T>;
}

pub struct TaskRepo<T: sqlx::Database> {
    db_pool: sqlx::pool::Pool<T>,
}

#[async_trait]
impl<T: sqlx::Database> Repository<Model> for TaskRepo<T> {
    async fn save(&self, new: Model) -> anyhow::Result<Model> {
        Ok(sqlx::query!(
            r#"
            INSERT INTO tasks (task_id, description, create_date, due_date, assignee)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING task_id
            "#,
            self.task_id,
            self.description,
            self.create_date,
            self.due_date,
            self.assignee
        )
        .fetch_one(&self.db_pool)
        .await?)
    }

    async fn retrieve_by_id(&self, id: Uuid) -> anyhow::Result<Model> {
        let task = sqlx::query_as!(
            Task,
            r#"
            SELECT task_id, description, create_date, due_date, assignee
            FROM tasks
            WHERE task_id = $1
            "#,
            task_id
        )
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(task)
    }

    async fn delete_by_id(&self, id: Uuid) -> anyhow::Result<Model> {
        Ok(sqlx::query_as!(
            Task,
            r#"
            DELETE FROM tasks
            WHERE task_id = $1
            RETURNING task_id, description, create_date, due_date, assignee
            "#,
            task_id
        )
        .fetch_optional(&self.db_pool)
        .await?)
    }
}
impl<T: sqlx::Database> TaskRepo<T> {
    pub fn new(pool: sqlx::Pool<T>) -> Self {
        Self { db_pool: pool }
    }
}
