use uuid::Uuid;

use crate::protocols::{ble_uuid_from_u16, ProtocolMatch};

pub const TUYA_SERVICE_SHORT: u16 = 0x1910;
pub const TUYA_NOTIFY_SHORT: u16 = 0x2b10;
pub const TUYA_WRITE_SHORT: u16 = 0x2b11;

pub fn tuya_service_uuid() -> Uuid {
    ble_uuid_from_u16(TUYA_SERVICE_SHORT)
}

pub fn tuya_notify_uuid() -> Uuid {
    ble_uuid_from_u16(TUYA_NOTIFY_SHORT)
}

pub fn tuya_write_uuid() -> Uuid {
    ble_uuid_from_u16(TUYA_WRITE_SHORT)
}

pub fn detect_from_name(name: Option<&str>) -> Option<ProtocolMatch> {
    let name = name?;
    name.trim()
        .eq_ignore_ascii_case("TY")
        .then(|| ProtocolMatch {
            protocol: "tuya-ble".to_string(),
            confidence: "candidate".to_string(),
            evidence: vec![
                "Geraetename TY: schwacher Hinweis aus PARKSIDE-Scan; GATT-Pruefung erforderlich"
                    .to_string(),
            ],
        })
}

pub fn detect_from_services(
    service_uuids: &[Uuid],
    characteristic_uuids: &[Uuid],
) -> Option<ProtocolMatch> {
    let service_found = service_uuids
        .iter()
        .any(|uuid| *uuid == tuya_service_uuid());
    let notify_found = characteristic_uuids
        .iter()
        .any(|uuid| *uuid == tuya_notify_uuid());
    let write_found = characteristic_uuids
        .iter()
        .any(|uuid| *uuid == tuya_write_uuid());

    if !service_found && !notify_found && !write_found {
        return None;
    }

    let mut evidence = Vec::new();
    if service_found {
        evidence.push("GATT-Service 0x1910 gefunden".to_string());
    }
    if notify_found {
        evidence.push("Characteristic 0x2B10 gefunden".to_string());
    }
    if write_found {
        evidence.push("Characteristic 0x2B11 gefunden".to_string());
    }

    let confidence = if service_found && notify_found && write_found {
        "likely"
    } else {
        "candidate"
    };

    Some(ProtocolMatch {
        protocol: "tuya-ble".to_string(),
        confidence: confidence.to_string(),
        evidence,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_full_tuya_candidate_set_as_likely() {
        let detected = detect_from_services(
            &[tuya_service_uuid()],
            &[tuya_notify_uuid(), tuya_write_uuid()],
        )
        .expect("match");

        assert_eq!(detected.confidence, "likely");
        assert_eq!(detected.evidence.len(), 3);
    }

    #[test]
    fn does_not_detect_without_known_uuids() {
        assert!(detect_from_services(&[], &[]).is_none());
    }
}
