use super::{intent::Intent, params::Params};
use serde::de::StdError;
use std::fmt::{Display, Formatter};

pub struct SuccessReport {
    intent: Intent,
    params: Params,
    description: String,
}
type Outcome = Result<SuccessReport, ExecutionRouteErr>;
#[derive(Debug)]
pub enum ExecutionRouteErr {
    InvalidIntentParamPairing {
        attempted_intent: Intent,
        attempted_params: Params,
    },
}
impl Display for ExecutionRouteErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl StdError for ExecutionRouteErr {}
pub async fn resolve(intent: Intent, params: Params) -> Outcome {
    match (intent, params) {
        (
            Intent::CreateNewTask,
            Params::CreateNewTask {
                description: _,
                due_date: _,
                assignee: _,
            },
        ) => todo!(),
        (Intent::ModifyExistingTask, Params::ModifyExistingTask { .. }) => todo!(),
        (Intent::DeleteTask, Params::DeleteTask { .. }) => todo!(),
        (Intent::QueryTasks, Params::QueryTasksParams { .. }) => todo!(),
        (intent, mispaired_params) => Err(ExecutionRouteErr::InvalidIntentParamPairing {
            attempted_intent: intent,
            attempted_params: mispaired_params,
        }),
    }
}
