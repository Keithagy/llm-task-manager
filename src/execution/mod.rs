use crate::domain::task::model::{DisplayableTaskVec, PartialTask, Task};
use crate::input::parsing_pipeline_steps::intent::Intent;
use crate::input::parsing_pipeline_steps::params;
use crate::telegram_bot;
use chrono::Utc;
use either::Either;
use serde::de::StdError;
use std::fmt::{Display, Formatter};

pub struct SuccessReport<T: Display> {
    intent: Intent,
    params: params::Extraction,
    outcome: T,
}
impl<T: Display> SuccessReport<T> {
    pub fn formatted_string(&self) -> String {
        format!(
            "Done || [Intent] {} || [Params] {} || [Outcome] {} ",
            self.intent, self.params, self.outcome
        )
    }
}
impl<T: Display> telegram_bot::Describe for SuccessReport<T> {
    fn describe(&self) -> impl core::fmt::Display {
        self.formatted_string()
    }
}

pub type Result<T> = std::result::Result<T, ExecutionErr>;
#[derive(Debug)]
pub enum ExecutionErr {
    InvalidIntentParamPairing {
        attempted_intent: Intent,
        attempted_params: params::Extraction,
    },
    TaskCreationError {
        e: anyhow::Error,
    },
    TaskModificationError {
        e: anyhow::Error,
    },
    TaskDeletionError {
        e: anyhow::Error,
    },
    TaskRetrievalError {
        e: anyhow::Error,
    },
}
impl Display for ExecutionErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl StdError for ExecutionErr {}
pub async fn resolve<S: crate::domain::task::service::TaskDataFlows>(
    intent: Intent,
    params: params::Extraction,
    task_data_flows: S,
) -> Result<SuccessReport<Either<Task, DisplayableTaskVec>>> {
    let outcome = match (intent.clone(), params.clone()) {
        (Intent::CreateNewTask, params::Extraction::CreateNewTask { found }) => {
            let created_task = task_data_flows
                .create_new_task(PartialTask {
                    id: None,
                    description: found.description,
                    create_date: Some(Utc::now()),
                    due_date: found.due_date,
                    assignee: found.assignee,
                })
                .await
                .map_err(|e| ExecutionErr::TaskCreationError { e })?;
            Ok(Either::Left(created_task))
        }
        (Intent::ModifyExistingTask, params::Extraction::ModifyExistingTask { found }) => {
            let modified_task = task_data_flows
                .modify_existing_task(found)
                .await
                .map_err(|e| ExecutionErr::TaskModificationError { e })?;
            Ok(Either::Left(modified_task))
        }
        (Intent::DeleteTask, params::Extraction::DeleteTask { found }) => {
            let deleted_task = task_data_flows
                .delete_existing_task(found)
                .await
                .map_err(|e| ExecutionErr::TaskDeletionError { e })?;
            Ok(Either::Left(deleted_task))
        }
        (Intent::QueryTasks, params::Extraction::QueryTasksParams { found }) => {
            let retrieved_tasks = task_data_flows
                .retrieve_tasks(found)
                .await
                .map_err(|e| ExecutionErr::TaskRetrievalError { e })?;
            Ok(Either::Right(DisplayableTaskVec::from(retrieved_tasks)))
        }

        (intent, mispaired_params) => Err(ExecutionErr::InvalidIntentParamPairing {
            attempted_intent: intent,
            attempted_params: mispaired_params,
        }),
    }?;
    Ok(SuccessReport {
        intent,
        params,
        outcome,
    })
}
