use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub struct Measurement {
    pub timestamp_utc: DateTime<Utc>,
    pub device_id: String,
    pub kind: String,
    pub value: serde_json::Value,
    pub unit: Option<String>,
}
