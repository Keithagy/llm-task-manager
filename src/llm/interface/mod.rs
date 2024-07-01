use anyhow::Error;
use async_trait::async_trait;
use serde::de::StdError;
use std::fmt::{Display, Formatter};

#[async_trait]
pub trait LLMClient<T> {
    async fn prompt(&self, prompt: &str) -> anyhow::Result<T>;
    async fn prompt_system_customized(
        &self,
        prompt: &str,
        customize_system_prompt: &str,
    ) -> anyhow::Result<T>;
}

#[derive(Debug)]
pub enum LLMError {
    NoChoicesGenerated,
    EmptyResponse,
}
impl Display for LLMError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
impl StdError for LLMError {}
impl From<Error> for LLMError {
    fn from(_value: Error) -> Self {
        LLMError::NoChoicesGenerated
    }
}
