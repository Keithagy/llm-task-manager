use super::super::interface::LLMClient;
use crate::llm::interface::LLMError;
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    ChatCompletionResponseFormat, ChatCompletionResponseFormatType,
    CreateChatCompletionRequestArgs,
};
use async_openai::Client as InnerClient;
use async_trait::async_trait;

pub struct Client {
    inner_client: InnerClient<OpenAIConfig>,
    system_prompt: String,
}
#[async_trait]
impl LLMClient<String> for Client {
    async fn prompt(&self, prompt: &str) -> anyhow::Result<String> {
        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo")
            .n(1)
            .response_format(ChatCompletionResponseFormat {
                r#type: ChatCompletionResponseFormatType::JsonObject,
            })
            .messages(vec![
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(&self.system_prompt)
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt)
                    .build()?
                    .into(),
            ])
            .build()?;
        let chat_completion_response = self.inner_client.chat().create(request).await?;
        let completion_choice = chat_completion_response
            .choices
            .first()
            .take()
            .ok_or(LLMError::NoChoicesGenerated)?;
        Ok(completion_choice
            .clone()
            .message
            .content
            .ok_or(LLMError::NoChoicesGenerated)?)
    }
}

static DEFAULT_SYSTEM_PROMPT: &str = "You are a helpful AI Assistant";
impl Client {
    /// Requires setting of OPENAI_API_KEY env var
    pub fn new(system_prompt: Option<String>) -> Self {
        let inner_client = InnerClient::new();
        Self {
            inner_client,
            system_prompt: system_prompt
                .or(Some(DEFAULT_SYSTEM_PROMPT.to_string()))
                .expect("use default system prompt if none provided"),
        }
    }
}
