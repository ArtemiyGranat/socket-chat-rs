use std::net::SocketAddr;

#[derive(Clone)]
pub struct Client {
    pub username: String,
    pub client_addr: SocketAddr,
}

// TODO: Use client to avoid many function arguments
impl Client {
    pub fn new(username: String, client_addr: SocketAddr) -> Self {
        Self {
            username,
            client_addr,
        }
    }
}