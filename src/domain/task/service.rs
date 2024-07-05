use super::model::{PartialTask, Task, TaskFieldFilter};
use crate::db::interface::Repository;
use anyhow::bail;
use async_trait::async_trait;
use uuid::Uuid;

pub struct Service<R: Repository<Task>> {
    repo: R,
}
impl<R: Repository<Task>> Service<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
    async fn retrieve_task_by_id(&self, id: Uuid) -> anyhow::Result<Task> {
        self.repo.retrieve_by_id(id).await
    }
}

#[async_trait]
impl<R: Repository<Task> + Sync> TaskDataFlows for Service<R> {
    async fn create_new_task(&self, mut fields: PartialTask) -> anyhow::Result<Task> {
        fields.id = Some(Uuid::new_v4());
        let new_task = Task::try_from(fields)?;
        self.repo.save(new_task).await
    }

    async fn modify_existing_task(&self, fields: PartialTask) -> anyhow::Result<Task> {
        // TODO: Need to think about how to look up existing task -- current approach of looking up by uuid isn't feasible
        if let None = fields.id {
            bail!("No task id specified for modification operation")
        }
        let existing_task = self
            .retrieve_task_by_id(fields.id.expect("task id presence already checked"))
            .await?;
        self.repo.save(fields.apply_partial(existing_task)).await
    }

    async fn delete_existing_task(&self, fields: PartialTask) -> anyhow::Result<Task> {
        // TODO: Need to think about how to look up existing task -- current approach of looking up by uuid isn't feasible
        if let None = fields.id {
            bail!("No task id specified for delete operation")
        }

        self.repo
            .delete_by_id(fields.id.expect("task id presence already checked"))
            .await
    }

    async fn retrieve_tasks(&self, filter: TaskFieldFilter) -> anyhow::Result<Vec<Task>> {
        todo!()
    }
}

#[async_trait]
pub trait TaskDataFlows {
    async fn create_new_task(&self, fields: PartialTask) -> anyhow::Result<Task>;
    async fn modify_existing_task(&self, fields: PartialTask) -> anyhow::Result<Task>;
    async fn delete_existing_task(&self, fields: PartialTask) -> anyhow::Result<Task>;
    async fn retrieve_tasks(&self, filter: TaskFieldFilter) -> anyhow::Result<Vec<Task>>;
}
