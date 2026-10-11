use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::protocols::ProtocolMatch;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterInfo {
    pub name: String,
    pub address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManufacturerData {
    pub company_id: u16,
    pub company_id_hex: String,
    pub data_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceData {
    pub service_uuid: String,
    pub data_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BleDeviceRecord {
    pub id: String,
    pub name: Option<String>,
    pub address: Option<String>,
    pub address_type: Option<String>,
    pub rssi: Option<i16>,
    pub tx_power_level: Option<i16>,
    pub manufacturer_data: Vec<ManufacturerData>,
    pub advertised_services: Vec<String>,
    pub service_data: Vec<ServiceData>,
    pub possible_profiles: Vec<ProtocolMatch>,
    pub last_seen_utc: DateTime<Utc>,
}

impl BleDeviceRecord {
    pub fn profile_summary(&self) -> String {
        if self.possible_profiles.is_empty() {
            "keine Kandidaten".to_string()
        } else {
            self.possible_profiles
                .iter()
                .map(|profile| format!("{}:{}", profile.protocol, profile.confidence))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }

    pub fn matches_user_selector(&self, selector: &str) -> bool {
        let needle = normalize_selector(selector);
        normalize_selector(&self.id) == needle
            || self
                .address
                .as_deref()
                .is_some_and(|address| normalize_selector(address) == needle)
            || self
                .name
                .as_deref()
                .is_some_and(|name| normalize_selector(name) == needle)
    }
}

pub fn normalize_selector(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn device_matching_is_stable_for_ids_names_and_addresses() {
        let record = BleDeviceRecord {
            id: "Device {ABC DEF}".to_string(),
            name: Some("Parkside Akku".to_string()),
            address: Some("AA:BB:CC:DD:EE:FF".to_string()),
            address_type: None,
            rssi: None,
            tx_power_level: None,
            manufacturer_data: Vec::new(),
            advertised_services: Vec::new(),
            service_data: Vec::new(),
            possible_profiles: Vec::new(),
            last_seen_utc: Utc::now(),
        };

        assert!(record.matches_user_selector(" Device {ABC DEF} "));
        assert!(record.matches_user_selector("aa:bb:cc:dd:ee:ff"));
        assert!(record.matches_user_selector("parkside akku"));
        assert!(!record.matches_user_selector("other"));
    }
}
