use std::{future::Future, pin::Pin};

use super::model::{FieldFilter, PartialTask, Task};
use crate::db::interface::Repository;
use anyhow::bail;
use chrono::{DateTime, Utc};
use uuid::Uuid;

type TaskCreator<'a> = Box<
    dyn Fn(
            String,
            DateTime<Utc>,
            String,
        ) -> Pin<Box<dyn Future<Output = anyhow::Result<Task>> + Send + 'a>>
        + Send
        + Sync
        + 'a,
>;
type TaskEditor<'a> = Box<
    dyn Fn(PartialTask) -> Pin<Box<dyn Future<Output = anyhow::Result<Task>> + Send + 'a>>
        + Send
        + Sync
        + 'a,
>;
type TaskDeletor<'a> = Box<
    dyn Fn(PartialTask) -> Pin<Box<dyn Future<Output = anyhow::Result<Task>> + Send + 'a>>
        + Send
        + Sync
        + 'a,
>;
// TODO: Pending further review of how to handle task retrieval
type TaskRetriever<'a> = Box<
    dyn Fn(FieldFilter) -> Pin<Box<dyn Future<Output = anyhow::Result<Vec<Task>>> + Send + 'a>>
        + Send
        + Sync
        + 'a,
>;

type TaskDataFlows<'a> = (
    TaskCreator<'a>,
    TaskEditor<'a>,
    TaskDeletor<'a>,
    TaskRetriever<'a>,
);
pub fn inject_task_management_dependencies<'a, R>(repo: &'a R) -> TaskDataFlows<'a>
where
    R: Repository<Task> + 'a + Sync,
{
    (
        Box::new(
            move |description: String, due_date: DateTime<Utc>, assignee: String| {
                Box::pin(
                    async move { create_new_task(repo, &description, due_date, &assignee).await },
                )
            },
        ),
        Box::new(move |updated_task_fields: PartialTask| {
            Box::pin(async move { modify_existing_task(repo, updated_task_fields).await })
        }),
        Box::new(move |to_delete: PartialTask| {
            Box::pin(async move { delete_existing_task(repo, to_delete).await })
        }),
        Box::new(move |filter: FieldFilter| {
            Box::pin(async move { retrieve_tasks(repo, filter).await })
        }),
    )
}

async fn retrieve_tasks(
    _repo: &impl Repository<Task>,
    _filter: FieldFilter,
) -> anyhow::Result<Vec<Task>> {
    todo!()
}

async fn delete_existing_task(
    repo: &impl Repository<Task>,
    to_delete: PartialTask,
) -> anyhow::Result<Task> {
    // TODO: Need to think about how to look up existing task -- current approach of looking up by uuid isn't feasible
    if let None = to_delete.id {
        bail!("No task id specified for delete operation")
    }

    Ok(repo
        .delete_by_id(to_delete.id.expect("task id presence already checked"))
        .await?)
}

async fn modify_existing_task(
    repo: &impl Repository<Task>,
    updated_task_fields: PartialTask,
) -> anyhow::Result<Task> {
    // TODO: Need to think about how to look up existing task -- current approach of looking up by uuid isn't feasible
    if let None = updated_task_fields.id {
        bail!("No task id specified for modification operation")
    }
    let existing_task = retrieve_task_by_id(
        repo,
        updated_task_fields
            .id
            .expect("task id presence already checked"),
    )
    .await?;
    Ok(repo
        .save(updated_task_fields.apply_partial(existing_task))
        .await?)
}
async fn retrieve_task_by_id(repo: &impl Repository<Task>, id: Uuid) -> anyhow::Result<Task> {
    repo.retrieve_by_id(id).await
}
async fn create_new_task(
    repo: &impl Repository<Task>,
    description: &str,
    due_date: DateTime<Utc>,
    assignee: &str,
) -> anyhow::Result<Task> {
    let new_task = Task {
        id: Uuid::new_v4(),
        description: description.to_string(),
        create_date: Utc::now(),
        due_date,
        assignee: assignee.to_string(),
    };
    Ok(repo.save(new_task).await?)
}
