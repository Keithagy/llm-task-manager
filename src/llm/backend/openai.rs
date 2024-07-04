use crate::llm::interface;
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
impl interface::LLMClient<String> for Client {
    async fn prompt(&self, prompt: &str) -> anyhow::Result<String> {
        Self::prompt_inner(prompt, &self.system_prompt, &self.inner_client).await
    }

    async fn prompt_system_customized(
        &self,
        prompt: &str,
        customize_system_prompt: &str,
    ) -> anyhow::Result<String> {
        let extended_system_prompt = format!("{}\n{}", customize_system_prompt, self.system_prompt);
        Self::prompt_inner(prompt, &extended_system_prompt, &self.inner_client).await
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
    async fn prompt_inner(
        prompt: &str,
        system_prompt: &str,
        client: &InnerClient<OpenAIConfig>,
    ) -> anyhow::Result<String> {
        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-3.5-turbo")
            .n(1)
            .response_format(ChatCompletionResponseFormat {
                r#type: ChatCompletionResponseFormatType::JsonObject,
            })
            .messages(vec![
                ChatCompletionRequestSystemMessageArgs::default()
                    .content(system_prompt)
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(prompt)
                    .build()?
                    .into(),
            ])
            .build()?;
        let chat_completion_response = client.chat().create(request).await?;
        let completion_choice = chat_completion_response
            .choices
            .first()
            .take()
            .ok_or(interface::LLMError::NoChoicesGenerated)?;
        Ok(completion_choice
            .clone()
            .message
            .content
            .ok_or(interface::LLMError::NoChoicesGenerated)?)
    }
}
