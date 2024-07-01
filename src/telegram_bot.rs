use core::fmt;

pub const TELEGRAM_BOT_API_KEY: &str = "7011341978:AAE0gWDYiKkRHJYEgPXibP48itZiMuzcKAE";

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
