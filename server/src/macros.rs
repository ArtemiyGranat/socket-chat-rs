#[macro_export]
macro_rules! conn_message {
    ($username:expr) => {{
        format!("{} has been connected to the server", $username)
    }};
}

#[macro_export]
macro_rules! disc_message {
    ($username:expr) => {{
        format!("{} has been disconnected from the server", $username)
    }};
}

#[macro_export]
macro_rules! response_to_json {
    ($status_code:expr, $message:expr) => {{
        let response = serde_json::json!({ "type": "response", "status_code": $status_code, "message": $message });
        response.to_string()
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
        request.to_string()
    }}
}
