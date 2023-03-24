use std::collections::HashMap;
use tokio::net::{TcpListener, unix::SocketAddr};

pub const MAX_CONNECTIONS: usize = 10;
pub const SERVER_ADDRESS: &str = "localhost:8080";

pub struct Server {
    clients_list: HashMap<SocketAddr, String>,
    listener: TcpListener,
}

impl Server {
    pub async fn new() -> Self {
        let listener = match TcpListener::bind(SERVER_ADDRESS).await {
            Ok(listener) => listener,
            Err(_) => {
                eprintln!("[ERROR] Could not bind the server to this address");
                std::process::exit(1)
            }
        };
        Self {
            clients_list: HashMap::new(),
            listener
        }
    }

    pub fn listener(&self) -> TcpListener {
        self.listener
    }

    pub fn clients_list(&self) -> HashMap<SocketAddr, String> {
        self.clients_list
    }
}