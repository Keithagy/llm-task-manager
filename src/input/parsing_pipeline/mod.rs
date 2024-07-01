use super::parsing_pipeline_steps::{execution, intent, params};
use crate::llm::interface::LLMClient;
pub async fn from_text(
    text: &str,
    llm_client: &impl LLMClient<String>,
) -> anyhow::Result<execution::SuccessReport> {
    let intent = intent::identify(llm_client, text).await?;
    let params = params::extract(llm_client, &intent, text).await?;
    let outcome = execution::resolve(intent, params).await?;
    Ok(outcome)
}
