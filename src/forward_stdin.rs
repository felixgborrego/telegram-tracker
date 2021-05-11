use std::io::{self, BufRead};

use futures::executor::block_on;
use log::info;
use rtdlib::types::*;
//use rtdlib::*;
use telegram_client::api::aasync::AsyncApi;
use telegram_client::api::Api;

const TELEGRAM_START_MSG: &str = "TELEGRAM BOT<<<";
/// Read stdin and forward to channel
pub fn forward_stdin_to_channel_id(api: Api, chat_id: i64) {
    info!("Listening stdin to forward to channel... {}", chat_id);
    let api = AsyncApi::new(api);
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let msg_to_send = line.unwrap().trim().to_owned();
        if msg_to_send.starts_with(TELEGRAM_START_MSG) {
            let msg_to_send = msg_to_send.replace(TELEGRAM_START_MSG, "");
            let result = block_on(send_msg(&api, chat_id, msg_to_send));
        }
    }
}

async fn send_msg(api: &AsyncApi, chat_id: i64, msg_to_send: String) {
    let msg_content = InputMessageContent::InputMessageText(
        InputMessageText::builder()
            .text(FormattedText::builder().text(msg_to_send).build())
            .clear_draft(true)
            .build(),
    );

    let msg: SendMessage = SendMessage::builder()
        .chat_id(chat_id)
        .input_message_content(msg_content)
        .build();

    api.send_message(msg).await.unwrap();
}
