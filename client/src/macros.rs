#[macro_export]
macro_rules! request_to_json {
    ($method:expr, $body:expr) => {{
        let request = match $method {
            "SendMessage" | "LogInUsername" =>
                serde_json::json!({ "type": "request_c2s", "method": $method, "body": $body }),
            "LogInPassword" | "RegisterUsername" | "MessageRead" | "GetHistory" => unimplemented!(),
            &_ => unreachable!()
        };
        request.to_string()
    }}
}
