use std::net::SocketAddr;
use crate::Result;

#[derive(Clone)]
pub struct Config {
    pub server_address: String,
    pub min_username_len: usize,
    pub max_username_len: usize,
    pub min_message_len: usize,
    pub max_message_len: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_address: "0.0.0.0:8080".to_string(),
            // TODO: Add regex for username and message to avoid invalid data
            min_username_len: 1,
            max_username_len: 20,
            min_message_len: 1,
            max_message_len: 256,
        }
    }
}

impl Config {
    pub fn is_valid_username(&self, username: Option<&str>, client_addr: SocketAddr) -> Result<bool> {
        if let Some(username) = username {
            Ok((self.min_username_len..=self.max_username_len).contains(&username.len()))
        } else {
            Err(format!("Invalid request from {}", client_addr).into())
        }
    }

    pub fn is_valid_message(&self, message: Option<&str>, client_addr: SocketAddr) -> Result<bool> {
        if let Some(message) = message {
            Ok((self.min_message_len..=self.max_message_len).contains(&message.trim().len()))
        } else {
            Err(format!("Invalid request from {}", client_addr).into())
        }
    }
}
