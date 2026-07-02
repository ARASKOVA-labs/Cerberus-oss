use teloxide::{prelude::*, utils::command::BotCommands};
use std::sync::Arc;
use tokio::sync::Mutex;
use cerberus_memory::StateDB;
use anyhow::Result;

pub mod whatsapp;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "start a new passive mission.")]
    Mission(String),
}

pub struct GatewayState {
    pub db: Arc<Mutex<StateDB>>,
}

pub async fn run_telegram_bot(bot_token: String, db: Arc<Mutex<StateDB>>) -> Result<()> {
    let bot = Bot::new(bot_token);
    
    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<Command>()
                .endpoint(answer_command),
        )
        .branch(dptree::endpoint(handle_message));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![db])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
        
    Ok(())
}

async fn answer_command(bot: Bot, msg: Message, cmd: Command, db: Arc<Mutex<StateDB>>) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
        }
        Command::Mission(objective) => {
            let session_id = msg.chat.id.to_string();
            {
                let db_lock = db.lock().await;
                let _ = db_lock.create_session(&session_id, "telegram");
            }
            bot.send_message(msg.chat.id, format!("Starting mission: {}", objective)).await?;
            // Real implementation would invoke Cerberus kernel here
        }
    };
    Ok(())
}

async fn handle_message(bot: Bot, msg: Message, db: Arc<Mutex<StateDB>>) -> ResponseResult<()> {
    if let Some(text) = msg.text() {
        let session_id = msg.chat.id.to_string();
        {
            let db_lock = db.lock().await;
            let _ = db_lock.create_session(&session_id, "telegram");
            let _ = db_lock.insert_message(&session_id, "user", text);
        }
        bot.send_message(msg.chat.id, "I recorded that in memory.").await?;
    }
    Ok(())
}
