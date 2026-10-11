use btleplug::api::{CharPropFlags, Characteristic, Service};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GattCharacteristicInfo {
    pub service_uuid: String,
    pub characteristic_uuid: String,
    pub properties: Vec<String>,
}

pub fn characteristic_info(characteristic: &Characteristic) -> GattCharacteristicInfo {
    GattCharacteristicInfo {
        service_uuid: characteristic.service_uuid.to_string(),
        characteristic_uuid: characteristic.uuid.to_string(),
        properties: property_names(characteristic.properties),
    }
}

pub fn service_uuids(services: &[Service]) -> Vec<Uuid> {
    let mut uuids = services
        .iter()
        .map(|service| service.uuid)
        .collect::<Vec<_>>();
    uuids.sort();
    uuids
}

pub fn characteristic_uuids(services: &[Service]) -> Vec<Uuid> {
    let mut uuids = services
        .iter()
        .flat_map(|service| service.characteristics.iter().map(|ch| ch.uuid))
        .collect::<Vec<_>>();
    uuids.sort();
    uuids
}

pub fn property_names(flags: CharPropFlags) -> Vec<String> {
    let known = [
        (CharPropFlags::BROADCAST, "broadcast"),
        (CharPropFlags::READ, "read"),
        (
            CharPropFlags::WRITE_WITHOUT_RESPONSE,
            "write_without_response",
        ),
        (CharPropFlags::WRITE, "write"),
        (CharPropFlags::NOTIFY, "notify"),
        (CharPropFlags::INDICATE, "indicate"),
        (
            CharPropFlags::AUTHENTICATED_SIGNED_WRITES,
            "authenticated_signed_writes",
        ),
        (CharPropFlags::EXTENDED_PROPERTIES, "extended_properties"),
    ];

    let mut names = known
        .iter()
        .filter_map(|(flag, name)| flags.contains(*flag).then_some((*name).to_string()))
        .collect::<Vec<_>>();
    if names.is_empty() {
        names.push("none".to_string());
    }
    names
}

pub fn can_read(characteristic: &Characteristic) -> bool {
    characteristic.properties.contains(CharPropFlags::READ)
}

pub fn can_subscribe(characteristic: &Characteristic) -> bool {
    characteristic.properties.contains(CharPropFlags::NOTIFY)
        || characteristic.properties.contains(CharPropFlags::INDICATE)
}
