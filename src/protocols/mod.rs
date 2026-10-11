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
    } else if let Some(profile) = tuya::detect_from_name(name) {
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

    #[test]
    fn ty_advertisement_is_only_a_candidate() {
        let profiles = detect_from_advertisement(Some("TY"), &[]);
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].protocol, "tuya-ble");
        assert_eq!(profiles[0].confidence, "candidate");
        assert!(detect_from_gatt(&[], &[]).is_empty());
        assert!(detect_from_advertisement(Some("TY-other"), &[]).is_empty());
        assert!(detect_from_advertisement(None, &[]).is_empty());
    }

    #[test]
    fn uuid_evidence_takes_precedence_over_ty_name() {
        let profiles = detect_from_advertisement(Some("TY"), &[tuya::tuya_service_uuid()]);
        assert_eq!(profiles.len(), 1);
        assert!(profiles[0].evidence[0].contains("0x1910"));
    }
}
