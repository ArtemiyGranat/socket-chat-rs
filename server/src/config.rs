use crate::Result;
use regex::Regex;
use std::net::SocketAddr;

#[derive(Clone)]
pub struct Config {
    pub server_address: String,
    pub min_username_len: usize,
    pub max_username_len: usize,
    pub min_message_len: usize,
    pub max_message_len: usize,
    username_regex: Regex,
    message_regex: Regex,
}

impl Default for Config {
    fn default() -> Self {
        let (min_username_len, max_username_len) = (4, 20);
        let (min_message_len, max_message_len) = (1, 256);
        Self {
            server_address: "0.0.0.0:8080".to_string(),
            min_username_len,
            max_username_len,
            min_message_len,
            max_message_len,
            username_regex: Regex::new(&format!(
                "[A-Za-z\\s]{{{},{}}}",
                min_username_len, max_username_len
            ))
            .unwrap(),
            message_regex: Regex::new(&format!(
                "[A-Za-z\\s]{{{},{}}}",
                min_message_len, max_message_len
            ))
            .unwrap(),
        }
    }
}

impl Config {
    pub fn is_valid_username(
        &self,
        username: Option<&str>,
        client_addr: SocketAddr,
    ) -> Result<bool> {
        if let Some(username) = username {
            Ok(self.username_regex.is_match(&username))
        } else {
            Err(format!("Invalid request from {}", client_addr).into())
        }
    }

    pub fn is_valid_message(&self, message: Option<&str>, client_addr: SocketAddr) -> Result<bool> {
        if let Some(message) = message {
            Ok(self.message_regex.is_match(&message.trim()))
        } else {
            Err(format!("Invalid request from {}", client_addr).into())
        }
    }
}
