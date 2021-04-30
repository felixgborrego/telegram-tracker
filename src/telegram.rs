use std::fs::File;

use colored::Colorize;
use log::{debug, error, info, warn};
use rtdlib::types::*;
use rtdlib::types::MessageContent::MessageText;
use telegram_client::api::Api;
use telegram_client::client::Client;

mod tgfn;
mod thelp;

fn on_new_message_in_room(msg: &String, chat_id: i64, sender_user_id: i64) {
    let line_msg = str::replace(msg, "\n", "; ");
    println!("### chat {}/{}, msg==> {}", chat_id, sender_user_id, line_msg);
}

pub fn start(phone: String, telegram_api_id: String, telegram_api_hash: String) {
    let mut client = config();
    let listener = client.listener();

    listener.on_update_authorization_state(move |(api, update)| {
        let state = update.authorization_state();
        state.on_wait_tdlib_parameters(|_| {
            api.set_tdlib_parameters(SetTdlibParameters::builder().parameters(
                TdlibParameters::builder()
                    .use_test_dc(false)
                    .database_directory("telegram_data")
                    .use_message_database(false)
                    //.use_secret_chats(true)
                    .api_id(toolkit::number::as_i64(&telegram_api_id).unwrap())
                    .api_hash(&telegram_api_hash)
                    .system_language_code("en")
                    .device_model("Android")
                    .system_version("Unknown")
                    .application_version(env!("CARGO_PKG_VERSION"))
                    .enable_storage_optimizer(true)
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
            warn!("Logging out");
        });
        state.on_closing(|_| {
            //let mut have_authorization = have_authorization.lock().unwrap();
            //*have_authorization = false;
            warn!("Closing");
        });
        state.on_closed(|_| {
            debug!("Closed");
        });
        Ok(())
    });

    listener.on_update_connection_state(|(_, update)| {
        let state = update.state();
        state
            .on_waiting_for_network(|_| {
                debug!("waiting for network");
            })
            .on_connecting_to_proxy(|_| {
                debug!("connecting to proxy");
            })
            .on_connecting(|_| {
                debug!("connecting...");
            })
            .on_updating(|_| {
                info!("updating...");
            })
            .on_ready(|_| info!("📡 Connection ready! Listening..."));
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
            _ => thelp::unknown(code, &message),
        };
        Ok(())
    });

    listener.on_ok(|_| {
        debug!("OK");
        Ok(())
    });

    listener.on_update_new_chat(|(_, update)| {
        let chat = update.chat();
        debug!(
            "Receive new chat, title: '{}', data: {}",
            chat.title(),
            chat.to_json().expect("Can't serialize json")
        );
        Ok(())
    });

    listener.on_update_user_status(|(_, update)| {
        debug!(
            "User [{}] status is {:?}",
            update.user_id(),
            update.status()
        );
        Ok(())
    });

    listener.on_update_new_message(|(_, update)| {
        let message = update.message();
        if message.is_outgoing() {
            return Ok(());
        }
        let content = message.content();
        content.on_message_text(|_| {
            debug!("Receive text message ");
        });

        content.on_message_video(|_| {
            debug!("Receive video message");
        });

        debug!(
            "Receive new message, from: '{:?}', data: {}",
            message.sender_user_id(),
            message.to_json().expect("Can't serialize json")
        );

        let mut msg_text = "".to_string();

        if let MessageText(text) = message.content() {
            msg_text.push_str(text.text().text());
        }

        if let Some(message_photo) = content.as_message_photo() {
            msg_text.push_str(message_photo.caption().text());
        }

        on_new_message_in_room(&msg_text, message.chat_id(), message.sender_user_id());
        Ok(())
    });

    listener.on_update_chat_last_message(|(_, update)| {
        debug!(
            "Chat last message: {}, data: {}",
            update.chat_id(),
            update
                .last_message()
                .clone()
                .map_or("None".to_string(), |v| v
                    .to_json()
                    .expect("Can't serialize json"))
        );
        Ok(())
    });

    listener.on_update_user(|(_, _)| {
        debug!("Update user");
        Ok(())
    });
    listener.on_update_have_pending_notifications(|(_, _)| {
        debug!("on_update_have_pending_notifications");
        Ok(())
    });
    listener.on_update_unread_chat_count(|(_, _)| {
        debug!("on_update_unread_chat_count");
        Ok(())
    });

    listener.on_update_selected_background(|_| Ok(()));

    client.daemon("Telegram-tracker").unwrap();
}

// Configure client
fn config() -> Client {
    // Log File
    let log_file = toolkit::path::root_dir().join("telegram_logs.log");
    if log_file.exists() {
        std::fs::remove_file(&log_file).unwrap();
    }
    File::create(&log_file).expect("Failed create log file");
    let api = Api::default();

    Client::set_log_verbosity_level(2).unwrap(); // Only 0 error messages,  2 waring, 5 all
    Client::set_log_file_path(log_file.to_str());

    let mut client = Client::new(api.clone());
    client.warn_unregister_listener(false); // No show errors for unregistered listeners

    client
}
