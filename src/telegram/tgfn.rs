use colored::Colorize;
use rtdlib::types::*;

use crate::telegram::thelp;
use log::{debug, info};
use telegram_client::api::aevent::EventApi;

pub fn type_phone_number(api: &EventApi) {
    let input = thelp::typed();
    api.set_authentication_phone_number(
        SetAuthenticationPhoneNumber::builder()
            .phone_number(&input)
            .build(),
    )
    .unwrap();
    debug!(
        "Set phone number [{}] {}",
        input.green(),
        "(If you copy log to anywhere, don't forget hide your phone number)".red()
    );
}

pub fn type_authentication_code(api: &EventApi) {
    let code = thelp::typed();
    api.check_authentication_code(CheckAuthenticationCode::builder().code(&code))
        .unwrap();
    info!("Set authentication code: {}, login...", code);
}

pub fn type_and_register(api: &EventApi) {
    let first_name = thelp::typed_with_message("Please input first name:");
    let last_name = thelp::typed_with_message("Please input last name:");
    debug!("You name is {} {}", first_name, last_name);
    api.register_user(
        RegisterUser::builder()
            .first_name(first_name)
            .last_name(last_name)
            .build(),
    )
    .unwrap();
}
