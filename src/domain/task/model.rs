use std::fmt::{self, Display};

use chrono::{DateTime, Utc};
use partial_derive::Partial;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Partial, Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    #[serde(with = "uuid::serde::simple")]
    pub id: Uuid,
    pub description: String,
    pub create_date: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub assignee: String,
}
impl Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Task: {} || Assignee: {} || Due Date: {}",
            self.description, self.assignee, self.due_date
        )
    }
}

#[derive(Clone)]
pub struct DisplayableTaskVec(Vec<Task>);

impl IntoIterator for DisplayableTaskVec {
    type Item = Task;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl Display for DisplayableTaskVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // FIXME
        let mut iter = self.clone().into_iter();
        if let Some(first) = iter.next() {
            write!(f, "{}", first)?;
            for task in iter {
                write!(f, ", {}", task)?;
            }
        }
        Ok(())
    }
}
impl From<Vec<Task>> for DisplayableTaskVec {
    fn from(value: Vec<Task>) -> Self {
        Self(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskFieldFilter {
    pub field: TaskField,
    pub sql_query: CheckedSqlQuery,
}
impl Default for TaskFieldFilter {
    fn default() -> Self {
        Self {
            field: TaskField::Id,
            sql_query: CheckedSqlQuery(String::from("")),
        }
    }
}

// NOTE: this is distinct from `&str` because it denotes that some SQL validation step has been
// cleared prior to being passed to the database
// FIXME: due consideration to be given to stopping LLM from generating security-compromising
// queries, as well as queries doing anything that isn't reading
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CheckedSqlQuery(pub String);

pub enum ParamTransformErr {
    /// You are trying to convert Params into a Model where you shouldn't be (e.g. Delete)
    WrongAccessContext,
}
use crate::input::parsing_pipeline_steps::params::{Extraction, PartialCreateNewTask};
impl TryFrom<Extraction> for PartialTask {
    type Error = ParamTransformErr;
    fn try_from(value: Extraction) -> Result<Self, Self::Error> {
        match value {
            Extraction::CreateNewTask {
                found:
                    PartialCreateNewTask {
                        description,
                        due_date,
                        assignee,
                    },
            } => Ok(PartialTask {
                id: None,
                description,
                create_date: Some(Utc::now()),
                due_date,
                assignee,
            }),
            _ => Err(Self::Error::WrongAccessContext),
        }
    }
}
