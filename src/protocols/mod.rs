pub mod parkside;
pub mod tuya;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolMatch {
    pub protocol: String,
    pub confidence: String,
    pub evidence: Vec<String>,
}

pub fn detect_from_advertisement(name: Option<&str>, service_uuids: &[Uuid]) -> Vec<ProtocolMatch> {
    let mut matches = Vec::new();
    if let Some(profile) = parkside::detect_from_name(name) {
        matches.push(profile);
    }
    if let Some(profile) = tuya::detect_from_services(service_uuids, &[]) {
        matches.push(profile);
    }
    matches
}

pub fn detect_from_gatt(
    service_uuids: &[Uuid],
    characteristic_uuids: &[Uuid],
) -> Vec<ProtocolMatch> {
    let mut matches = Vec::new();
    if let Some(profile) = tuya::detect_from_services(service_uuids, characteristic_uuids) {
        matches.push(profile);
    }
    matches
}

pub fn ble_uuid_from_u16(short: u16) -> Uuid {
    Uuid::from_u128(0x00000000_0000_1000_8000_00805f9b34fb | ((short as u128) << 96))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_ble_uuid_correctly() {
        assert_eq!(
            ble_uuid_from_u16(0x1910).to_string(),
            "00001910-0000-1000-8000-00805f9b34fb"
        );
    }
}
