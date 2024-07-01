use super::intent::Intent;
use crate::data::task::{FieldFilter as TaskFieldFilter, PartialModel as PartialTaskModel};
use anyhow::Error;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

use crate::llm::interface::LLMClient;
use chrono::{DateTime, Utc};
use serde::de::StdError;

#[derive(Serialize, Deserialize, Debug)]
pub enum Params {
    CreateNewTask {
        description: String,
        due_date: DateTime<Utc>,
        assignee: String,
    },
    ModifyExistingTask {
        #[serde(with = "uuid::serde::simple")]
        task_id: Uuid,
        fields_to_modify: PartialTaskModel,
    },
    DeleteTask {
        #[serde(with = "uuid::serde::simple")]
        task_id: Uuid,
    },
    QueryTasksParams {
        query_filters: Vec<TaskFieldFilter>,
    },
}

#[derive(Debug)]
pub enum ParamExtractErr {
    IncompleteParams,
    Deserialization { e: serde_json::Error },
    LLMFailed,
}
impl Display for ParamExtractErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
impl StdError for ParamExtractErr {}
impl From<Error> for ParamExtractErr {
    fn from(_value: Error) -> Self {
        ParamExtractErr::LLMFailed
    }
}
pub async fn extract(
    llm_client: &impl LLMClient<String>,
    intent: &Intent,
    text_message_content: &str,
) -> Result<Params, ParamExtractErr> {
    let schema = intent.get_params_schema();
    let generate_system_prompt = || -> String {
        let mut system_prompt = String::from("You will be provided text content to parse for input parameters, per the following schema:");
        system_prompt.push_str(serde_json::to_string_pretty(&schema).unwrap().as_str());
        system_prompt
    };
    let llm_response = llm_client
        .prompt_system_customized(text_message_content, generate_system_prompt().as_str())
        .await?;
    let parse_result: serde_json::Result<Params> = serde_json::from_str(&llm_response);
    match parse_result {
        Ok(params) => Ok(params),
        Err(e) => Err(ParamExtractErr::Deserialization { e }),
    }
}
