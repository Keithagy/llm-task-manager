use crate::domain::task::model::PartialTask;
use crate::input::parsing_pipeline_steps::params::Params;
use crate::llm::interface::LLMClient;
use anyhow::Error;
use schemars::schema::RootSchema;
use schemars::schema_for_value;
use serde::de::StdError;
use serde::Serialize;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Debug)]
pub enum Intent {
    CreateNewTask,
    ModifyExistingTask,
    DeleteTask,
    QueryTasks,
}
impl Intent {
    // TODO: We are currently passing in a concrete instance, but we actually need a
    // json representation of the struct definition i.e the schema
    pub fn get_params_schema(&self) -> RootSchema {
        match self {
            Intent::CreateNewTask => schema_for_value!(Params::CreateNewTask {
                description: "".to_string(),
                due_date: Default::default(),
                assignee: "".to_string(),
            }),
            Intent::ModifyExistingTask => schema_for_value!(Params::ModifyExistingTask {
                task_id: Default::default(),
                fields_to_modify: PartialTask {
                    ..Default::default()
                },
            }),
            Intent::DeleteTask => schema_for_value!(Params::DeleteTask {
                task_id: Default::default(),
            }),
            Intent::QueryTasks => schema_for_value!(Params::QueryTasksParams {
                query_filters: vec![],
            }),
        }
    }
}

#[derive(Debug)]
pub enum IntentIdErr {
    NoApparentIntent,
    LLMFailed,
}

impl Display for IntentIdErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl StdError for IntentIdErr {}
impl From<Error> for IntentIdErr {
    fn from(_value: Error) -> Self {
        Self::LLMFailed
    }
}

pub async fn identify(
    llm_client: &impl LLMClient<String>,
    text_message_content: &str,
) -> Result<Intent, IntentIdErr> {
    let llm_response = llm_client.prompt(text_message_content).await?;
    match llm_response.as_str() {
        "create new task" => Ok(Intent::CreateNewTask),
        "modify existing task" => Ok(Intent::ModifyExistingTask),
        "delete task" => Ok(Intent::DeleteTask),
        "query tasks" => Ok(Intent::QueryTasks),
        "no apparent intent" => Err(IntentIdErr::NoApparentIntent),
        _ => Err(IntentIdErr::LLMFailed),
    }
}
