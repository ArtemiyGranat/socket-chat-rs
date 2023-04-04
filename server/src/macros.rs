#[macro_export]
macro_rules! conn_message {
    ($username:expr) => {{
        let conn_message = format!("{} has been connected to the server\n", $username);
        conn_message
    }};
}

#[macro_export]
macro_rules! disc_message {
    ($username:expr) => {{
        let disc_message = format!("{} has been disconnected from the server\n", $username);
        disc_message
    }};
}

#[macro_export]
macro_rules! print_message {
    ($username:expr, $data:expr) => {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S");
        print!("[{}] [{}] {}", now, $username, $data)
    };
}

#[macro_export]
macro_rules! message_to_json {
    ($username:expr, $data:expr) => {{
        let now = Local::now().format("%Y-%m-%d %H:%M:%S %z").to_string();
        let json = serde_json::json!({ "type": "message", "sender": $username, "data": $data, "date": now });
        format!("{}\n", json)
    }};
}

#[macro_export]
macro_rules! response_to_json {
    ($response:expr) => {{
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S %z").to_string();
        let json = serde_json::json!({ "type": "response", "data": $response, "date": now });
        format!("{}\n", json)
    }};
}
