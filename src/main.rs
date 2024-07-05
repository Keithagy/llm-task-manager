#![feature(associated_type_defaults)]
#![feature(impl_trait_in_fn_trait_return)]

mod db;
mod domain;
mod execution;
mod input;
mod llm;
mod telegram_bot;
mod transcription;

use sqlx::postgres::PgPool;
use sqlx::Postgres;
use std::env;
use std::sync::Arc;
use telegram_bot::{Context, Describe};
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::{Dispatcher, UpdateFilterExt};
use teloxide::net::Download;
use teloxide::prelude::*;
use teloxide::types::{File as TelegramFile, MediaKind, Message, MessageKind};
use teloxide::{dptree, Bot};
use tokio::fs::{self, File as TokioFile};
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    log::info!("Starting dialogue bot...");
    // initialize tracing
    tracing_subscriber::fmt::init();

    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;
    let task_repo = db::task::Repository::new(pool);
    let task_service = domain::task::service::Service::new(task_repo);
    let llm_client = llm::backend::openai::Client::new(Some(String::from(
        r#"You are managing a task assignment system. Provide all response as JSON objects only.
In the event of errors or uncertain outcome, return the empty JSON object."#,
    )));
    let whisper_client = transcription::backend::openai::WhisperClient::new();
    let ctx = telegram_bot::Context {
        transcription_client: whisper_client,
        llm_client,
        chat_log: vec![],
        task_data_flows: task_service,
    };

    let telegram_bot = teloxide::Bot::new(telegram_bot::TELEGRAM_BOT_API_KEY);
    Dispatcher::builder(
        telegram_bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<telegram_bot::InteractionSteps>, telegram_bot::InteractionSteps>()
            .branch(dptree::case![telegram_bot::InteractionSteps::ReceiveInput].endpoint(receive_input::<transcription::backend::openai::WhisperClient, llm::backend::openai::Client, domain::task::service::Service<db::task::Repository<Postgres>>>))
            .branch(dptree::case![telegram_bot::InteractionSteps::ValidateParams { input, intent, params }].endpoint(validate_params::<transcription::backend::openai::WhisperClient, llm::backend::openai::Client, domain::task::service::Service<db::task::Repository<Postgres>>>))
    )
    .dependencies(dptree::deps![
        ctx,
        InMemStorage::<telegram_bot::InteractionSteps>::new()
    ])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
    Ok(())
}

async fn receive_input<
    T: transcription::interface::TranscriptionClient,
    L: llm::interface::LLMClient<String>,
    S: domain::task::service::TaskDataFlows,
>(
    bot: Bot,
    msg: Message,
    dialogue: telegram_bot::Dialogue,
    ctx: Arc<Context<T, L, S>>,
) -> anyhow::Result<()> {
    bot.send_message(msg.chat.id, "Gotcha! One second...")
        .await?;
    let extracted_input = match &msg.kind {
        MessageKind::Common(msg_common) => match &msg_common.media_kind {
            MediaKind::Text(content) => Some(content.text.clone()),
            MediaKind::Voice(content) => transcribe_voice_input(&bot, content, &msg, &ctx).await?,
            other_media_kinds => {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "I don't know what to do with {} messages. Sorry!",
                        other_media_kinds.describe()
                    ),
                )
                .await?;
                None
            }
        },
        _ => None,
    };
    Ok(if let Some(content) = extracted_input {
        match input::parsing_pipeline::from_text(&content, &ctx.llm_client).await {
            Ok((intent, params)) => {
                match execution::resolve(intent, params, &ctx.task_data_flows).await {
                    Ok(success_report) => {
                        bot.send_message(msg.chat.id, success_report.formatted_string())
                            .await;
                    }
                    Err(e) => {
                        match e {
                            execution::ExecutionErr::InvalidIntentParamPairing {
                                attempted_intent,
                                attempted_params,
                            } => {
                                // WARN: Getting to this branch indicates a bug in Intent-to-Param
                                // matching logic
                                bot.send_message(msg.chat.id, format!( r#"Oops, you've uncovered a bug! Let the dev know. Thanks and sorry!
Intent-Param-Mismatch
Intent: {}
Params: {}
"#, attempted_intent, attempted_params)).await;
                            }
                            // WARN: Everything below indicates some arbitrary service-layer error.
                            // Use service-layer logs to debug that.
                            execution::ExecutionErr::TaskCreationError { e }
                            | execution::ExecutionErr::TaskModificationError { e }
                            | execution::ExecutionErr::TaskDeletionError { e }
                            | execution::ExecutionErr::TaskRetrievalError { e } => {
                                bot.send_message(
                                    msg.chat.id,
                                    format!(
                                        r#"Your operation failed. Here's the error:
{}"#,
                                        e
                                    ),
                                )
                                .await;
                            }
                        };
                    }
                }
            }
            Err(e) => {
                match e {
                    // NOTE: Defensive / Corrective routing should happen here
                    input::parsing_pipeline::InputParseErr::IntentErr(intent_err) => {
                        bot.send_message(
                            msg.chat.id,
                            format!(
                                r#"I didn't really understand what you were looking for there.
                                Error: {}"#,
                                intent_err
                            ),
                        )
                        .await;
                        // TODO: retry intent extraction prompt?
                    }
                    input::parsing_pipeline::InputParseErr::ParamsErr(
                        pe,
                        intent,
                        extraction_attempt,
                    ) => {
                        bot.send_message(
                            msg.chat.id,
                            format!(
                                r#"I think I got what you were looking for, but I'm not sure you gave me everything I needed to what you're asking.
                                Error: {}
                                Intent: {}
                                Params Found: {:#?}
                                "#,
                                pe,
                                intent,
                                extraction_attempt
                            ),
                        ).await;
                        // TODO: based on missing params, ask the user to provide in a subsequent
                        // message, and first generate schema based on missing fields and then run
                        // transcription / extraction against that
                        dialogue
                            .update(telegram_bot::InteractionSteps::ValidateParams {
                                input: content,
                                intent,
                                params: extraction_attempt,
                            })
                            .await;
                    }
                    input::parsing_pipeline::InputParseErr::OtherErr(oe) => {
                        bot.send_message(
                            msg.chat.id,
                            format!(
                                r#"Something went wrong there... Could you try providing an input again?
                                Error: {}"#,
                                oe
                            ),
                        )
                        .await;
                    }
                }
            }
        }
    })
}

async fn validate_params<
    T: transcription::interface::TranscriptionClient,
    L: llm::interface::LLMClient<String>,
    S: domain::task::service::TaskDataFlows,
>(
    bot: Bot,
    msg: Message,
    dialogue: telegram_bot::Dialogue,
    ctx: Arc<Context<T, L, S>>,
    input: String,
    intent: input::parsing_pipeline_steps::intent::Intent,
    params: Option<input::parsing_pipeline_steps::params::Extraction>,
) -> anyhow::Result<()> {
    // Again, safe concurrent access to the shared context
    let task_data_flows = &ctx.task_data_flows;

    // Handler logic here...

    Ok(())
}

async fn transcribe_voice_input<
    T: transcription::interface::TranscriptionClient,
    L: llm::interface::LLMClient<String>,
    S: domain::task::service::TaskDataFlows,
>(
    bot: &Bot,
    content: &teloxide::types::MediaVoice,
    msg: &Message,
    ctx: &Context<T, L, S>,
) -> Result<Option<String>, anyhow::Error> {
    let TelegramFile { path, .. } = bot.get_file(&content.voice.file.id).await?;
    let tmp_file_name = format!("/tmp/transcribe/{}.ogg", msg.id);
    let mut dst = TokioFile::create(&tmp_file_name).await?;
    bot.download_file(&path, &mut dst).await?;
    let mut download_as_buffer = Vec::<u8>::new();
    dst.read_to_end(&mut download_as_buffer).await?;
    let transcript = ctx
        .transcription_client
        .transcribe(download_as_buffer)
        .await?;
    fs::remove_file(&tmp_file_name).await?;
    Ok(Some(transcript))
}
