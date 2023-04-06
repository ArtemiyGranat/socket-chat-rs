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
macro_rules! response_to_json {
    ($status_code:expr, $message:expr) => {{
        let response = serde_json::json!({ "type": "response", "status_code": $status_code, "message": $message });
        format!("{}\n", response)
    }}
}

#[macro_export]
macro_rules! request_to_json {
    ($method:expr, $body:expr) => {{
        let request = match $method {
            "Connection" | "SendMessage" =>
                serde_json::json!({ "type": "request_s2c", "method": $method, "body": $body}),
            "MessageRead" => unimplemented!(),
            &_ => unreachable!()
        };
        format!("{}\n", request)
    }}
}
