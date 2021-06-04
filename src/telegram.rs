use std::fs::File;
use std::thread;

use chrono;
use chrono::{DateTime, NaiveDateTime, Utc};
use colored::Colorize;
use log::{debug, error, info, warn};
use rtdlib::types::MessageContent::*;
use rtdlib::types::*;
use telegram_client::api::aevent::EventApi;
use telegram_client::api::Api;
use telegram_client::client::Client;

use crate::{tgfn, thelp, TelegramMessage};
use std::sync::mpsc::SyncSender;

fn on_new_message(
    event_info: String,
    message: &Message,
    only_channel_id: &Option<i64>,
    message_sender: SyncSender<TelegramMessage>,
) {
    if message.is_outgoing() {
        debug!("Ignoring outgoing message ");
    }

    let mut msg_text = "".to_string();

    if let MessageText(text) = message.content() {
        msg_text.push_str(text.text().text());
    }
    if let MessageVideo(video) = message.content() {
        msg_text.push_str(video.caption().text());
    }
    if let MessagePhoto(photo) = message.content() {
        msg_text.push_str(photo.caption().text());
    }

    if let MessageDocument(doc) = message.content() {
        msg_text.push_str(doc.caption().text());
    }

    let msg_text = str::replace(&*msg_text, "\n", "; "); // one msg per line

    let naive = NaiveDateTime::from_timestamp(message.date(), 0);

    // Create a normal DateTime from the NaiveDateTime
    let sent_datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    let sender_id = match message.sender() {
        MessageSender::_Default(_) => -1,
        MessageSender::Chat(c) => c.chat_id(),
        MessageSender::User(u) => u.user_id(),
    };

    if *only_channel_id == Some(message.chat_id()) || only_channel_id.is_none() {
        let msg = TelegramMessage {
            event_info,
            msg_text,
            chat_id: message.chat_id(),
            message_id: message.id(),
            sender_id,
            sent_datetime,
        };
        message_sender.send(msg).unwrap();
    } else {
        info!(
            "Ignoring message chat: {}; message_id: {}, time:{:?}; event_info: {}, msg: {}",
            message.chat_id(),
            message.id(),
            sent_datetime,
            event_info,
            &msg_text
        );
    }
}

pub fn start(
    phone: String,
    telegram_api_id: String,
    telegram_api_hash: String,
    print_outgoing: bool,
    follow_channel: Option<i64>,
    message_sender: SyncSender<TelegramMessage>,
) -> Api {
    let (client, api) = config();
    thread::spawn(move || {
        start_telegram_tracking(
            client,
            phone,
            telegram_api_id,
            telegram_api_hash,
            print_outgoing,
            follow_channel,
            message_sender.clone(),
        );
    });
    api
}

fn start_telegram_tracking(
    mut client: Client,
    phone: String,
    telegram_api_id: String,
    telegram_api_hash: String,
    print_outgoing: bool,
    follow_channel: Option<i64>,
    message_sender: SyncSender<TelegramMessage>,
) {
    let listener = client.listener();
    let data_directory = format!("telegram_data-{}", follow_channel.unwrap_or(0));

    listener.on_update_authorization_state(move |(api, update)| {
        let state = update.authorization_state();
        state.on_wait_tdlib_parameters(|_| {
            api.set_tdlib_parameters(SetTdlibParameters::builder().parameters(
                TdlibParameters::builder()
                    .use_test_dc(false)
                    .database_directory(&data_directory)
                    .use_message_database(false)
                    .use_secret_chats(true)
                    .api_id(toolkit::number::as_i64(&telegram_api_id).unwrap())
                    .api_hash(&telegram_api_hash)
                    .system_language_code("en")
                    .device_model("Android")
                    .system_version("Unknown")
                    .application_version(env!("CARGO_PKG_VERSION"))
                    .enable_storage_optimizer(false)
                    .use_chat_info_database(false)
                    .files_directory(format!("{}/files", &data_directory))
                    .build()
            ).build()).unwrap();
            debug!("Set tdlib parameters");
        });
        state.on_wait_encryption_key(|_| {
            api.check_database_encryption_key(CheckDatabaseEncryptionKey::builder().build()).unwrap();
            debug!("Set encryption key");
        });
        state.on_wait_phone_number(|_| {
            api.set_authentication_phone_number(SetAuthenticationPhoneNumber::builder().phone_number(&phone).build()).unwrap();
            info!("Sent a Auth Key, waiting...");
        });
        state.on_wait_password(|_| {
            api.check_authentication_password(CheckAuthenticationPassword::builder()
                .password(thelp::typed_with_message(format!("{} {}", "Please type your telegram password:", "(If you copy log to anywhere, don't forget hide your password)".red())))
                .build()).unwrap();
            debug!("Set password *****");
        });
        state.on_wait_registration(|_| {
            thelp::tip("Welcome to use telegram");
            thelp::tip("Your phone number is not registered to telegram, please type your name. and register.");
            tgfn::type_and_register(api);
        });
        state.on_wait_code(|_| {
            thelp::tip("⚠️ Please type authentication code:");
            tgfn::type_authentication_code(api);
        });


        state.on_ready(|_| {
            info!("✅ Authorization ready!");
        });
        state.on_logging_out(|_| {
            //let mut have_authorization = have_authorization.lock().unwrap();
            //*have_authorization = false;
            info!("⚠️ Logging out");
        });
        state.on_closing(|_| {
            warn!("⚠️Closing");
        });
        state.on_closed(|_| {
            warn!("⚠️Closed");
        });
        Ok(())
    });

    listener.on_update_connection_state(move |(api, update)| {
        let state = update.state();
        state
            .on_waiting_for_network(|_| {
                info!("waiting for network");
            })
            .on_connecting_to_proxy(|_| {
                debug!("connecting to proxy");
            })
            .on_connecting(|_| {
                info!("connecting...");
            })
            .on_updating(|_| {
                info!("updating...");
            })
            .on_ready(|_| {
                info!("📡 Connection ready! Listening...");
                open_channel(&follow_channel, api)
            });
        Ok(())
    });

    listener.on_error(|(api, update)| {
        let code = update.code();
        let message = update.message();
        error!("ERROR [{}] {}", code, message);
        match code {
            8 => {
                thelp::tip(&message);
                thelp::tip("Please type telegram phone number");
                tgfn::type_phone_number(api);
            }
            400 => match &message[..] {
                "PHONE_NUMBER_INVALID" => {
                    thelp::tip(format!(
                        "{} {}",
                        "⚠️ Phone number invalid, please type a right phone number for telegram",
                        "(If you copy log to anywhere, don't forget hide your phone number)".red()
                    ));
                    tgfn::type_phone_number(api);
                }
                "PHONE_CODE_INVALID" | "PHONE_CODE_EMPTY" => {
                    thelp::tip("⚠️ Phone code invalid, please type an authentication code");
                    tgfn::type_authentication_code(api);
                }
                _ => {}
            },
            429 => thelp::wait_too_many_requests(api, &message),
            3 => {
                let result = api.get_chats(GetChats::builder().limit(100).build());
                info!(
                    "⚠️ Chat request not found, trying to refresh channels...{:?}",
                    result
                );
            }
            _ => thelp::unknown(code, &message),
        };
        Ok(())
    });

    listener.on_ok(|_| {
        //debug!("OK");
        Ok(())
    });

    listener.on_chat(move |(api, chat)| {
        info!("on_chat {:?}", chat);
        open_channel(&follow_channel, api);
        Ok(())
    });
    listener.on_update_new_chat(move |(api, update)| {
        let chat = update.chat();
        info!(
            "Receive new chat, title: '{}', id: {}, title: {}",
            chat.title(),
            chat.id(),
            chat.title()
        );

        if follow_channel == Some(chat.id()) {
            info!("📡 Found the required chat, opening...");
            let result = api.open_chat(OpenChat::builder().chat_id(chat.id()).build());
            info!("on_update_new_chat opening result: {:?}", result);
        }
        Ok(())
    });

    listener.on_update_user_status(move |(api, _)| {
        open_channel(&follow_channel, api);
        Ok(())
    });

    listener.on_update_delete_messages(move |(api, update)| {
        let chat_id = update.chat_id();
        info!("on_update_delete_messages chat_id {}", chat_id);
        open_channel(&follow_channel, api);
        Ok(())
    });

    listener.on_update_new_message(move |(_, update)| {
        let message = update.message();
        if message.is_outgoing() && print_outgoing == false {
            debug!("Ignoring outgoing message ");
            return Ok(());
        }
        on_new_message(
            "on_update_new_message".to_string(),
            message,
            &follow_channel,
            message_sender.clone(),
        );
        Ok(())
    });

    listener.on_update_chat_last_message(move |(_, _)| Ok(()));

    listener.on_update_have_pending_notifications(|(_, _)| Ok(()));

    listener.on_update_user(|(_, _)| Ok(()));

    listener.on_update_have_pending_notifications(|(_, _)| Ok(()));

    listener.on_update_unread_chat_count(|(_, _)| Ok(()));

    listener.on_update_selected_background(|_| Ok(()));

    client.daemon("Telegram-tracker").unwrap();
}

fn open_channel(follow_channel: &Option<i64>, api: &EventApi) {
    if let Some(channel_id) = &follow_channel {
        //info!("📡 Opening channel to follow...");
        let option_value: OptionValueBoolean = OptionValueBoolean::builder().value(true).build();
        api.set_option(
            SetOption::builder()
                .name("online")
                .value(OptionValue::Boolean(option_value)),
        )
        .unwrap();

        let _ = api.open_chat(OpenChat::builder().chat_id(*channel_id).build());
    }
}

// Configure client
fn config() -> (Client, Api) {
    // Log File
    let log_file = toolkit::path::root_dir().join("telegram_logs.log");
    if log_file.exists() {
        std::fs::remove_file(&log_file).unwrap();
    }
    File::create(&log_file).expect("Failed create log file");
    let api = Api::default();

    Client::set_log_verbosity_level(4).unwrap(); // Only 0 error messages,  2 waring, 5 all
    Client::set_log_file_path(log_file.to_str());

    let mut client = Client::new(api.clone());
    client.warn_unregister_listener(false); // No show errors for unregistered listeners

    (client, api)
}
