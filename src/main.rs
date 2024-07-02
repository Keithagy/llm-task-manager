#![feature(associated_type_defaults)]

mod db;
mod domain;
mod input;
mod llm;
mod telegram_bot;

use db::task::TaskRepo;
use domain::task::flow::inject_task_management_dependencies;
use llm::backend::openai;
use sqlx::postgres::PgPool;
use std::env;
use telegram_bot::Describe;
use teloxide::prelude::Requester;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let telegram_bot = teloxide::Bot::new(telegram_bot::TELEGRAM_BOT_API_KEY);
    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;
    let task_repo = TaskRepo::new(pool);
    let task_creator = inject_task_management_dependencies(&task_repo);

    teloxide::repl(
        telegram_bot,
        |bot: teloxide::Bot, msg: teloxide::prelude::Message| async move {
            let llm_client = openai::Client::new(Some(String::from(
                r#"You are managing a task assignment system. Provide all response as JSON objects only.
In the event of errors or uncertain outcome, return the empty JSON object."#,
            )));
            if let teloxide::types::MessageKind::Common(message_format) = msg.kind {
                match message_format.media_kind {
                    teloxide::types::MediaKind::Text(content) => {
                        input::parsing_pipeline::from_text(&content.text, &llm_client).await.expect("Parsing pipeline should succeed");
                        todo!()
                    }
                    // teloxide::types::MediaKind::Voice(_) => todo!(),
                    _ => {
                        bot.send_message(
                            msg.chat.id,
                            format!("I can't work with {}", message_format.media_kind.describe()),
                        )
                        .await?;
                    }
                }
            }
            Ok(())
        },
    )
    .await;
    Ok(())
}
