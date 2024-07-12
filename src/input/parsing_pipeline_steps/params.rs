use super::intent::Intent;
use crate::domain::task::model::{PartialTask, TaskFieldFilter};
use anyhow::Error;
use partial_derive::Partial;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::{Display, Formatter};

use crate::llm::interface::LLMClient;
use chrono::{DateTime, Utc};
use serde::de::StdError;

#[derive(Partial, Default, Debug, Clone, Serialize, Deserialize)]
pub struct CreateNewTask {
    pub description: String,
    pub due_date: DateTime<Utc>,
    pub assignee: String,
}

pub type ModifyExistingTask = PartialTask;
pub type DeleteTask = PartialTask;
pub type QueryTasks = TaskFieldFilter;

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Note that `found` in each variant can be incomplete, since it is likely that the user misses
/// out at least one param in the initial input.
pub enum Extraction {
    CreateNewTask { found: PartialCreateNewTask },
    ModifyExistingTask { found: ModifyExistingTask },
    DeleteTask { found: DeleteTask },
    QueryTasks { found: QueryTasks },
}

impl Default for Extraction {
    fn default() -> Self {
        Self::CreateNewTask {
            found: PartialCreateNewTask::default(),
        }
    }
}
impl Display for Extraction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum ExtractErr {
    Deserialization,
    LLMFailed,
}
impl Display for ExtractErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
impl StdError for ExtractErr {}
impl From<Error> for ExtractErr {
    fn from(_value: Error) -> Self {
        ExtractErr::LLMFailed
    }
}
pub async fn extract(
    llm_client: &impl LLMClient<String>,
    intent: &Intent,
    text_message_content: &str,
) -> Result<Extraction, ExtractErr> {
    let schema = intent.get_params_schema();
    let generate_system_prompt = || -> String {
        let mut system_prompt = String::from("You will be provided text content to parse for input parameters, per the following schema:");
        system_prompt.push_str(serde_json::to_string_pretty(&schema).unwrap().as_str());
        system_prompt
    };
    let llm_response = llm_client
        .prompt_system_customized(text_message_content, generate_system_prompt().as_str())
        .await?;
    let parse_result: serde_json::Result<Extraction> = serde_json::from_str(&llm_response);
    match parse_result {
        Ok(params) => Ok(params),
        Err(_) => Err(ExtractErr::Deserialization),
    }
}
