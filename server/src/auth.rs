use crate::{response_to_json, server::CONFIG, Result};
use futures::SinkExt;
use log::info;
use serde_json::Value;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, LinesCodec};

pub(crate) async fn registration(
    lines: &mut Framed<TcpStream, LinesCodec>,
    client_addr: SocketAddr,
) -> Result<String> {
    unimplemented!()
}

pub(crate) async fn authorize_user(
    lines: &mut Framed<TcpStream, LinesCodec>,
    client_addr: SocketAddr,
) -> Result<String> {
    let username = username(lines, client_addr).await;
    username
}

async fn username(
    lines: &mut Framed<TcpStream, LinesCodec>,
    client_addr: SocketAddr,
) -> Result<String> {
    loop {
        let mut username = None;
        let request = match lines.next().await {
            Some(Ok(request)) => request,
            Some(Err(e)) => return Err(format!("Invalid request from {client_addr}: {e}").into()),
            None => {
                return Err(format!("{client_addr} disconnected before entering username").into())
            }
        };

        // TODO: Handle it to prevent connections from the browser
        // IDEA: some ping-pong just after the connection
        let json_request: Value = serde_json::from_str(&request).unwrap();
        let (response, status_code) = match json_request.get("method").and_then(|v| v.as_str()) {
            Some("LogInUsername") => {
                username = json_request.get("body").and_then(|v| v.as_str());
                if CONFIG.is_valid_username(username, client_addr)? {
                    (response_to_json!(200, "OK"), 200)
                } else {
                    (response_to_json!(400, "InvalidUsername"), 400)
                }
            }
            _ => (response_to_json!(400, "BadRequest"), 400),
        };

        if let Err(e) = lines.send(&response).await {
            info!("Could not send a message to {client_addr}: {e}");
        }

        if status_code == 200 {
            return Ok(username.unwrap().to_string());
        }
    }
}

async fn password(
    lines: &mut Framed<TcpStream, LinesCodec>,
    client_addr: SocketAddr,
) -> Result<String> {
    loop {
        let mut password = None;
        let request = match lines.next().await {
            Some(Ok(request)) => request,
            Some(Err(e)) => return Err(format!("Invalid request from {client_addr}: {e}").into()),
            None => {
                return Err(format!("{client_addr} disconnected before entering password").into())
            }
        };

        // TODO: Handle it to prevent connections from the browser
        let json_request: Value = serde_json::from_str(&request).unwrap();
        let (response, status_code) = match json_request.get("method").and_then(|v| v.as_str()) {
            Some("LogInPassword") => {
                password = json_request.get("body").and_then(|v| v.as_str());
                // TODO: Handle password and work with database
                if CONFIG.is_valid_username(password, client_addr)? {
                    (response_to_json!(200, "OK"), 200)
                } else {
                    (response_to_json!(400, "InvalidUsername"), 400)
                }
            }
            _ => (response_to_json!(400, "BadRequest"), 400),
        };

        if let Err(e) = lines.send(&response).await {
            info!("Could not send a message to {client_addr}: {e}");
        }

        if status_code == 200 {
            return Ok(password.unwrap().to_string());
        }
    }
}