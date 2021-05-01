mod telegram;
use clap::{AppSettings, Clap};
use log::LevelFilter;
use simple_logger::SimpleLogger;

#[derive(Clap)]
#[clap(
    version = "1.0",
    author = "Felix G. Borrego K. <felix.g.borrego@gmail.com>"
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
}

fn main() {
    let opts: Opts = Opts::parse();

    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    println!(
        "Starting Telegram for: {}, api key: {}",
        opts.phone, opts.telegram_api_id
    );
    telegram::start(
        opts.phone,
        opts.telegram_api_id,
        opts.telegram_api_hash,
        opts.print_outgoing.parse().expect("It must be a bool"),
    );
}
