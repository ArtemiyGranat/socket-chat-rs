use chrono::{DateTime, Local};
use serde_json::Value;

#[derive(Clone)]
pub struct Message {
    pub data: String,
    pub sender: Option<String>,
    pub date: String,
}

impl Message {
    pub fn new(data: String, sender: Option<String>, date: String) -> Self {
        Self { data, sender, date }
    }

    pub fn from_json_value(value: Value) -> Self {
        let body = value.get("body").unwrap();
        let date = body
            .get("date")
            .map(|v| {
                let date = v.as_str().unwrap();
                let local_date = DateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S %z")
                    .unwrap()
                    .with_timezone(&Local);
                local_date.format("%d-%m-%Y %H:%M").to_string()
            })
            .unwrap();
        let sender = body
            .get("sender")
            .map(|v| v.as_str().unwrap().to_string());
        let data = body
            .get("data")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();
        Self { data, sender, date }
    }
}
