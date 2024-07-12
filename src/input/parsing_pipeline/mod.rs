use crate::llm::interface::LLMClient;

use super::parsing_pipeline_steps::{
    intent::{self, Intent, IntentIdErr},
    params::{self, ExtractErr, Extraction as ExtractedParams},
};
pub enum InputParseErr {
    IntentErr(IntentIdErr),
    ParamsErr(ExtractErr, Intent, Option<ExtractedParams>),
    OtherErr(anyhow::Error),
}
pub async fn from_text(
    text: &str,
    llm_client: &impl LLMClient<String>,
) -> Result<(Intent, ExtractedParams), InputParseErr> {
    let intent = intent::identify(llm_client, text)
        .await
        .map_err(InputParseErr::IntentErr)?;
    let params = params::extract(llm_client, &intent, text)
        .await
        .map_err(|e| {
            let clone = e.clone();
            match e {
                ExtractErr::Deserialization { .. } => {
                    InputParseErr::ParamsErr(clone, intent.clone(), None)
                }
                ExtractErr::LLMFailed => InputParseErr::ParamsErr(clone, intent.clone(), None),
            }
        })?;
    Ok((intent, params))
}
