use chrono::{DateTime, Utc};
use filter_by_field_derive::FilterByField;
use partial_derive::Partial;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(FilterByField, Partial, Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    #[serde(with = "uuid::serde::simple")]
    pub id: Uuid,
    pub description: String,
    pub create_date: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub assignee: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldFilter {
    pub field: TaskField,
    pub sql_query: CheckedSqlQuery,
}

// NOTE: this is distinct from `&str` because it denotes that some SQL validation step has been
// cleared prior to being passed to the database
// FIXME: due consideration to be given to stopping LLM from generating security-compromising
// queries, as well as queries doing anything that isn't reading
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckedSqlQuery(String);

pub enum ParamTransformErr {
    /// You are trying to convert Params into a Model where you shouldn't be (e.g. Delete)
    WrongAccessContext,
}
use crate::input::Params;
impl TryFrom<Params> for PartialTask {
    type Error = ParamTransformErr;
    fn try_from(value: Params) -> Result<Self, Self::Error> {
        match value {
            Params::CreateNewTask {
                description,
                due_date,
                assignee,
            } => Ok(PartialTask {
                id: None,
                description: Some(description),
                create_date: Some(Utc::now()),
                due_date: Some(due_date),
                assignee: Some(assignee),
            }),
            _ => Err(Self::Error::WrongAccessContext),
        }
    }
}
