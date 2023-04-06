use chrono::{DateTime, FixedOffset, Local};
use serde_json::{json, Value};

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
            .map(|data| {
                let date = data.as_str().unwrap();
                let utc_date: DateTime<FixedOffset> =
                    DateTime::parse_from_str(date, "%Y-%m-%d %H:%M:%S %z").unwrap();
                let local_date: DateTime<Local> = utc_date.into();
                json!(local_date.format("%d-%m-%Y %H:%M").to_string())
            })
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let sender = body
            .get("sender")
            .map(|data| data.as_str().unwrap().to_string());
        let data = body
            .get("data")
            .and_then(|data| data.as_str())
            .unwrap()
            .to_string();
        Self { data, sender, date }
    }
}
