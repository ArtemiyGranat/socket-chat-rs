#[macro_export]
macro_rules! request_to_json {
    ($method:expr, $body:expr) => {{
        let request = match $method {
            "SendMessage" => {
                serde_json::json!({ "type": "request_c2s", "method": $method, "body": $body })
            }
            "LogInUsername" => {
                serde_json::json!({ "type": "request_c2s", "method": $method, "body": $body })
            }
            "LogInPassword" => unimplemented!(),
            "RegisterUsername" => unimplemented!(),
            "MessageRead" => unimplemented!(),
            "GetHistory" => unimplemented!(),
            &_ => unreachable!()
        };
        format!("{}\n", request)
    }}
}
