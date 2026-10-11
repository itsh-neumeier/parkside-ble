use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RecorderError {
    #[error("E/A-Fehler beim Schreiben des Diagnoseprotokolls: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON-Serialisierung fehlgeschlagen: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    AdapterDetected,
    ScanStarted,
    DeviceDiscovered,
    ScanCompleted,
    ConnectionStarted,
    Connected,
    ServiceDiscovered,
    CharacteristicDiscovered,
    CharacteristicRead,
    NotificationReceived,
    ConnectionLost,
    Disconnected,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiagnosticEvent {
    pub timestamp_utc: DateTime<Utc>,
    pub event_type: EventKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub characteristic_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_hex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interpretation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl DiagnosticEvent {
    pub fn new(event_type: EventKind) -> Self {
        Self {
            timestamp_utc: Utc::now(),
            event_type,
            adapter: None,
            device_id: None,
            service_uuid: None,
            characteristic_uuid: None,
            raw_hex: None,
            interpretation: None,
            status: None,
            error: None,
            details: None,
        }
    }

    pub fn with_adapter(mut self, adapter: impl Into<String>) -> Self {
        self.adapter = Some(adapter.into());
        self
    }

    pub fn with_device(mut self, device_id: impl Into<String>) -> Self {
        self.device_id = Some(device_id.into());
        self
    }

    pub fn with_service(mut self, service_uuid: impl Into<String>) -> Self {
        self.service_uuid = Some(service_uuid.into());
        self
    }

    pub fn with_characteristic(mut self, characteristic_uuid: impl Into<String>) -> Self {
        self.characteristic_uuid = Some(characteristic_uuid.into());
        self
    }

    pub fn with_raw(mut self, data: &[u8]) -> Self {
        self.raw_hex = Some(hex::encode_upper(data));
        self
    }

    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }
}

pub struct JsonlRecorder {
    writer: BufWriter<File>,
}

impl JsonlRecorder {
    pub fn create(path: &Path) -> Result<Self, RecorderError> {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        Ok(Self {
            writer: BufWriter::new(file),
        })
    }

    pub fn write(&mut self, event: &DiagnosticEvent) -> Result<(), RecorderError> {
        serde_json::to_writer(&mut self.writer, event)?;
        self.writer.write_all(b"\n")?;
        self.writer.flush()?;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<(), RecorderError> {
        self.writer.flush()?;
        Ok(())
    }
}

pub fn details_from_pairs(pairs: &[(&str, String)]) -> Value {
    let mut object = serde_json::Map::new();
    for (key, value) in pairs {
        object.insert((*key).to_string(), json!(value));
    }
    Value::Object(object)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_jsonl_serializes_one_object_per_line() {
        let event = DiagnosticEvent::new(EventKind::CharacteristicRead)
            .with_device("id 1")
            .with_service("00001910-0000-1000-8000-00805f9b34fb")
            .with_characteristic("00002b10-0000-1000-8000-00805f9b34fb")
            .with_raw(&[0, 1, 0xab]);

        let json = serde_json::to_string(&event).expect("serialize");
        assert!(json.contains(r#""event_type":"characteristic_read""#));
        assert!(json.contains(r#""raw_hex":"0001AB""#));
        assert!(!json.contains('\n'));
    }
}
