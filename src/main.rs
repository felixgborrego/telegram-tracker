use clap::{AppSettings, Clap};
use log::LevelFilter;
use simple_logger::SimpleLogger;
use std::sync::mpsc::sync_channel;
use telegram_tracker::TelegramMessage;

mod telegram;
mod tgfn;
mod thelp;

#[derive(Clap)]
#[clap(
    version = "0.1.6",
    author = "Felix G. Borrego <felix.g.borrego@gmail.com>"
)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(long)]
    phone: String,
    #[clap(long)]
    telegram_api_id: String,
    #[clap(long)]
    telegram_api_hash: String,
    #[clap(long, default_value = "false")]
    print_outgoing: String,
    #[clap(long)]
    follow_channel_id: Option<String>,
}

fn main() {
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    let opts: Opts = Opts::parse();

    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        .with_module_level("telegram_client::api", LevelFilter::Info)
        .with_module_level("telegram_client::handler", LevelFilter::Off)
        .init()
        .unwrap();

    let channel_id = opts.follow_channel_id.map(|id| {
        format!("-100{}", id)
            .parse::<i64>()
            .expect("Expected a valid chat_id")
    });

    if let Some(id) = channel_id {
        println!(
            "Starting Telegram Tracker {} following channel  {} for: {}, api key: {}",
            VERSION, id, opts.phone, opts.telegram_api_id
        );
    } else {
        println!(
            "Starting Telegram Tracker {} for: {}, api key: {}",
            VERSION, opts.phone, opts.telegram_api_id
        );
    }

    let (message_sender, message_receiver) = sync_channel(10);
    let _ = telegram::start(
        opts.phone,
        opts.telegram_api_id,
        opts.telegram_api_hash,
        opts.print_outgoing.parse().expect("It must be a bool"),
        channel_id,
        message_sender,
    );

    println!("Stdin to Telegram is disabled.");
    for message in message_receiver {
        on_new_message_in_room(message);
    }
}

fn on_new_message_in_room(message: TelegramMessage) {
    println!(
        "### chat: {};sender_id: {};message_id: {};time: {:?};event_info: {}; msg:==> {}",
        message.chat_id,
        message.sender_id,
        message.message_id,
        message.sent_datetime.to_rfc3339(),
        message.event_info,
        message.msg_text
    );
}
