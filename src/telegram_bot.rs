use crate::{
    domain::task::service::TaskDataFlows,
    input::parsing_pipeline_steps::{intent::Intent, params::Extraction},
    llm, transcription,
};
use core::fmt;

pub const TELEGRAM_BOT_API_KEY: &str = "7011341978:AAE0gWDYiKkRHJYEgPXibP48itZiMuzcKAE";

use teloxide::dispatching::dialogue;
pub type Dialogue = dialogue::Dialogue<InteractionSteps, dialogue::InMemStorage<InteractionSteps>>;

#[derive(Clone, Default)]
pub enum InteractionSteps {
    #[default]
    ReceiveInput,
    ExtractIntent {
        input: String,
    },
    ExtractParams {
        input: String,
        intent: Intent,
    },
    ValidateParams {
        input: String,
        intent: Intent,
        params: Option<Extraction>,
    },
    NotifyExecute {
        intent: Intent,
        params: Extraction,
    },
    NotifyResult,
}

pub struct Context<
    T: transcription::interface::TranscriptionClient,
    L: llm::interface::LLMClient<String>,
    S: TaskDataFlows,
> {
    pub transcription_client: T,
    pub llm_client: L,
    pub task_data_flows: S,
    pub chat_log: Vec<String>,
}

pub trait Describe {
    fn describe(&self) -> impl fmt::Display;
}

impl Describe for teloxide::types::MediaKind {
    fn describe(&self) -> impl fmt::Display {
        match self {
            teloxide::types::MediaKind::Animation(_) => "animation",
            teloxide::types::MediaKind::Audio(_) => "audio",
            teloxide::types::MediaKind::Contact(_) => "contact",
            teloxide::types::MediaKind::Document(_) => "document",
            teloxide::types::MediaKind::Game(_) => "game",
            teloxide::types::MediaKind::Venue(_) => "venue",
            teloxide::types::MediaKind::Location(_) => "location",
            teloxide::types::MediaKind::Photo(_) => "photo",
            teloxide::types::MediaKind::Poll(_) => "poll",
            teloxide::types::MediaKind::Sticker(_) => "sticker",
            teloxide::types::MediaKind::Text(_) => "text",
            teloxide::types::MediaKind::Video(_) => "video",
            teloxide::types::MediaKind::VideoNote(_) => "videoNote",
            teloxide::types::MediaKind::Voice(_) => "voice",
            teloxide::types::MediaKind::Migration(_) => "migration",
        }
    }
}
