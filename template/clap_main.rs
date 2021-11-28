use clap::{App, AppSettings};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const BIN_NAME: &'static str = env!("CARGO_BIN_NAME");

fn main() {
    let args = App::new(BIN_NAME)
        .version(VERSION)
        .about("Help!
        ")
        .setting(AppSettings::ArgRequiredElseHelp)
        .get_matches();
}