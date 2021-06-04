use std::sync::mpsc::SyncSender;

use chrono::{DateTime, Utc};
use log::{debug, error, info};
use rtdlib::types::{FormattedText, InputMessageContent, SendMessage};
use rtdlib::types::InputMessageText;
use telegram_client::api::Api;

mod telegram;
mod tgfn;
mod thelp;

pub fn test(s: String) {
    println!("test {}", s);
}

#[derive(Debug, Clone)]
pub struct TelegramMessage {
    pub event_info: String,
    pub msg_text: String,
    pub chat_id: i64,
    pub message_id: i64,
    pub sender_id: i64,
    pub sent_datetime: DateTime<Utc>,
}

impl TelegramMessage {
    pub fn msg_lower_case(&self) -> String {
        self.msg_text.to_lowercase()
    }
}
#[derive(Debug, Clone)]
pub struct TelegramConfig {
    pub phone: String,
    pub telegram_api_id: String,
    pub telegram_api_hash: String,
    pub print_outgoing: bool,
    pub follow_channel: Option<i64>,
    pub send_notifications_to_channel: Option<i64>,
}

pub struct TelegramTrackerClient {
    api: Api,
    config: TelegramConfig,
}

impl TelegramTrackerClient {
    pub fn new(
        config: TelegramConfig,
        sender: SyncSender<TelegramMessage>,
    ) -> TelegramTrackerClient {
        let api = telegram::start(
            config.phone.to_owned(),
            config.telegram_api_id.to_owned(),
            config.telegram_api_hash.to_owned(),
            config.print_outgoing.to_owned(),
            config.follow_channel.to_owned(),
            sender,
        );
        TelegramTrackerClient { api, config }
    }

    pub fn send(&self, msg_to_send: &String) {
        info!("SENDING >> 💬 {}", msg_to_send);
        if let Some(chanel_id) = self.config.send_notifications_to_channel {
            let msg_content = InputMessageContent::InputMessageText(
                InputMessageText::builder()
                    .text(FormattedText::builder().text(msg_to_send).build())
                    .clear_draft(true)
                    .build(),
            );

            let msg: SendMessage = SendMessage::builder()
                .chat_id(chanel_id)
                .input_message_content(msg_content)
                .build();


            match self.api.send(msg) {
                Ok(ok) => debug!("Message forwarded to Telegram {:?}: {}  {}", ok,chanel_id, msg_to_send),
                Err(err) => error!("Unable forward to Telegram {:?} ", err),
            };
        }
    }
}
