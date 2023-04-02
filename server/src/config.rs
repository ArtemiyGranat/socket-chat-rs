#[derive(Clone)]
pub struct Config {
    pub server_address: String,
    pub max_connections: usize,
    pub min_username_len: usize,
    pub max_username_len: usize,
    pub min_message_len: usize,
    pub max_message_len: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_address: "0.0.0.0:8080".to_string(),
            max_connections: 10,
            min_username_len: 1,
            max_username_len: 20,
            min_message_len: 1,
            max_message_len: 256,
        }
    }
}
