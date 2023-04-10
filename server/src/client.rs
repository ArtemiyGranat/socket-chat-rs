use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::sync::{mpsc, Mutex};

pub struct Client {
    pub username: String,
    pub addr: SocketAddr,
    pub rx: mpsc::UnboundedReceiver<String>,
}

impl Client {
    pub async fn new(
        clients: &Arc<Mutex<HashMap<SocketAddr, mpsc::UnboundedSender<String>>>>,
        username: String,
        addr: SocketAddr,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        clients.lock().await.insert(addr, tx);

        Self { username, addr, rx }
    }
}
