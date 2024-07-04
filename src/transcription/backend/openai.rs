use crate::transcription::interface;
use async_openai::config::OpenAIConfig;
use async_openai::types::{AudioInput, AudioResponseFormat, CreateTranscriptionRequestArgs};
use async_openai::Client as InnerClient;
use async_trait::async_trait;

pub struct WhisperClient {
    inner_client: InnerClient<OpenAIConfig>,
}

#[async_trait]
impl interface::TranscriptionClient for WhisperClient {
    async fn transcribe(&self, audio_file_buf: Vec<u8>) -> anyhow::Result<String> {
        Self::transcribe_inner(&self.inner_client, audio_file_buf).await
    }
}

impl WhisperClient {
    /// Requires setting of OPENAI_API_KEY env var
    pub fn new() -> Self {
        let inner_client = InnerClient::new();
        Self { inner_client }
    }

    async fn transcribe_inner(
        client: &InnerClient<OpenAIConfig>,
        audio_file_buf: Vec<u8>,
    ) -> anyhow::Result<String> {
        let audio_input = AudioInput::from_vec_u8("tmp.mp3".to_string(), audio_file_buf);

        let request = CreateTranscriptionRequestArgs::default()
            .file(audio_input)
            .model("whisper-1")
            .response_format(AudioResponseFormat::Text)
            .build()?;

        let transcription = client.audio().transcribe(request).await?;

        Ok(transcription.text)
    }
}
