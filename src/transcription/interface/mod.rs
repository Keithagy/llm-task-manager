use async_trait::async_trait;

#[async_trait]
pub trait TranscriptionClient {
    async fn transcribe(&self, audio_file_buf: Vec<u8>) -> anyhow::Result<String>;
}
