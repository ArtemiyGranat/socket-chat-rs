use std::io::Write;

use chrono::Local;

const DEFAULT: &str = "\x1B[0m";
// TODO: Add the Data struct with username, date and data fields
// TODO: Color the username with green color
const GREEN: &str = "\x1B[32m";
const RED: &str = "\x1B[31m";
const BLUE: &str = "\x1B[34m";
const BOLD: &str = "\x1B[1m";

pub fn display_error(error: &str) {
    eprintln!("{}[ERROR] {}{}", RED, DEFAULT, error);
}

pub fn display_message(message: &str) {
    print!(
        "\r{}[{}] {}{}",
        BLUE,
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        DEFAULT,
        message
    );
}

pub fn display_message_waiting() {
    print!("{}[YOU] {}", BLUE, DEFAULT);
    std::io::stdout().flush().unwrap();
}