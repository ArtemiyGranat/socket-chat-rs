use chrono::Local;
use std::io::Write;
use termion::{color, style};

// TODO: Add the Data struct with username, date and data fields
// TODO: Color the username with green color

pub fn display_error(error: &str) {
    eprintln!("{}[ERROR] {}{}", color::Fg(color::Red), style::Reset, error);
}

pub fn display_message(message: &str) {
    std::io::stdout().flush().unwrap();
    print!(
        "\n\r{}[{}] {}{}",
        color::Fg(color::Blue),
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        style::Reset,
        message
    );
}

pub fn display_message_waiting() {
    print!("{}[YOU] {}", color::Fg(color::Blue), style::Reset);
    std::io::stdout().flush().unwrap();
}
