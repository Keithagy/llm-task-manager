use super::task::{Model as TaskModel, PartialModel as PartialTaskModel};
use super::Repository;
use anyhow::bail;
use chrono::{DateTime, Utc};
use uuid::Uuid;

type TaskCreator =
    fn(description: &str, due_date: DateTime<Utc>, assignee: &str) -> anyhow::Result<TaskModel>;
type TaskEditor = fn(updated_task_fields: PartialTaskModel) -> anyhow::Result<TaskModel>;
type TaskDeletor = fn(to_delete: PartialTaskModel) -> anyhow::Result<TaskModel>;

// TODO: Pending further review of how to handle task retrieval
type TaskRetriever = fn(lookup: PartialTaskModel) -> anyhow::Result<Vec<TaskModel>>;

type TaskDataFlows = (TaskCreator, TaskEditor, TaskDeletor, TaskRetriever);
pub fn inject_task_management_dependencies(repo: &impl Repository<TaskModel>) -> TaskDataFlows {
    (
        |description: &str, due_date: DateTime<Utc>, assignee: &str| async {
            create_new_task(repo, description, due_date, assignee).await?
        },
        |updated_task_fields: PartialTaskModel| async {
            modify_existing_task(repo, updated_task_fields).await?
        },
    )
}

async fn delete_existing_task(
    repo: &impl Repository<TaskModel>,
    to_delete: PartialTaskModel,
) -> anyhow::Result<TaskModel> {
    Ok(repo.delete_by_id(to_delete).await?)
}

async fn modify_existing_task(
    repo: &impl Repository<TaskModel>,
    updated_task_fields: PartialTaskModel,
) -> anyhow::Result<TaskModel> {
    // TODO: Need to think about how to look up existing task -- current approach of looking up by uuid isn't feasible
    if let None = updated_task_fields.id {
        bail!("No task id specified for modification operation")
    }
    let existing_task = retrieve_task_by_id(
        repo,
        updated_task_fields.id.expect("task id already checked"),
    )
    .await?;
    Ok(repo
        .save(updated_task_fields.apply_partial(existing_task))
        .await?)
}
async fn retrieve_task_by_id(
    repo: &impl Repository<TaskModel>,
    id: Uuid,
) -> anyhow::Result<TaskModel> {
    repo.retrieve_by_id(id).await
}
async fn create_new_task(
    repo: &impl Repository<TaskModel>,
    description: &str,
    due_date: DateTime<Utc>,
    assignee: &str,
) -> anyhow::Result<TaskModel> {
    let new_task = TaskModel {
        id: Uuid::new_v4(),
        description: description.to_string(),
        create_date: Utc::now(),
        due_date,
        assignee: assignee.to_string(),
    };
    Ok(repo.save(new_task).await?)
}
