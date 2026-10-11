use std::{collections::BTreeMap, time::Duration};

use anyhow::{anyhow, Context, Result};
use btleplug::{
    api::{Central, Manager as _, Peripheral as _, PeripheralProperties, ScanFilter},
    platform::{Adapter, Manager, Peripheral},
};
use chrono::Utc;
use tokio::time::sleep;
use uuid::Uuid;

use crate::{
    models::device::{BleDeviceRecord, ManufacturerData, ServiceData},
    protocols,
};

pub async fn scan(seconds: u64, _include_all: bool) -> Result<Vec<BleDeviceRecord>> {
    let adapter = first_adapter().await?;
    let adapter_name = adapter
        .adapter_info()
        .await
        .unwrap_or_else(|_| "<unbekannter Adapter>".to_string());

    println!("Starte BLE-Scan auf Adapter {adapter_name} fuer {seconds} Sekunden ...");
    adapter
        .start_scan(ScanFilter::default())
        .await
        .context("BLE-Scan konnte nicht gestartet werden. Ist Bluetooth aktiviert?")?;

    sleep(Duration::from_secs(seconds)).await;

    let peripherals = adapter
        .peripherals()
        .await
        .context("Gescannte Geraete konnten nicht gelesen werden")?;
    let _ = adapter.stop_scan().await;

    records_from_peripherals(peripherals).await
}

pub async fn first_adapter() -> Result<Adapter> {
    let manager = Manager::new()
        .await
        .context("Bluetooth-Manager konnte nicht initialisiert werden")?;
    let adapters = manager
        .adapters()
        .await
        .context("Bluetooth-Adapter konnten nicht gelesen werden")?;

    adapters
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("Kein Bluetooth-Adapter gefunden. Bitte Bluetooth aktivieren oder Adapter anschliessen."))
}

pub async fn records_from_peripherals(
    peripherals: Vec<Peripheral>,
) -> Result<Vec<BleDeviceRecord>> {
    let mut records = Vec::new();
    for peripheral in peripherals {
        match peripheral.properties().await {
            Ok(Some(properties)) => records.push(record_from_properties(&peripheral, properties)),
            Ok(None) => records.push(record_without_properties(&peripheral)),
            Err(error) => {
                tracing::warn!("Geraeteeigenschaften konnten nicht gelesen werden: {error}");
                records.push(record_without_properties(&peripheral));
            }
        }
    }

    records.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| right.rssi.cmp(&left.rssi))
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(records)
}

fn record_from_properties(
    peripheral: &Peripheral,
    properties: PeripheralProperties,
) -> BleDeviceRecord {
    let advertised_service_uuids = sorted_uuids(properties.services);
    let name = properties
        .local_name
        .or(properties.advertisement_name)
        .filter(|name| !name.trim().is_empty());
    let possible_profiles =
        protocols::detect_from_advertisement(name.as_deref(), &advertised_service_uuids);

    BleDeviceRecord {
        id: peripheral.id().to_string(),
        name,
        address: Some(properties.address.to_string()),
        address_type: properties
            .address_type
            .map(|address_type| format!("{address_type:?}")),
        rssi: properties.rssi,
        tx_power_level: properties.tx_power_level,
        manufacturer_data: manufacturer_data(properties.manufacturer_data),
        advertised_services: advertised_service_uuids
            .iter()
            .map(ToString::to_string)
            .collect(),
        service_data: service_data(properties.service_data),
        possible_profiles,
        last_seen_utc: Utc::now(),
    }
}

fn record_without_properties(peripheral: &Peripheral) -> BleDeviceRecord {
    BleDeviceRecord {
        id: peripheral.id().to_string(),
        name: None,
        address: None,
        address_type: None,
        rssi: None,
        tx_power_level: None,
        manufacturer_data: Vec::new(),
        advertised_services: Vec::new(),
        service_data: Vec::new(),
        possible_profiles: Vec::new(),
        last_seen_utc: Utc::now(),
    }
}

fn manufacturer_data(data: std::collections::HashMap<u16, Vec<u8>>) -> Vec<ManufacturerData> {
    let mut sorted = data.into_iter().collect::<BTreeMap<_, _>>();
    sorted
        .iter_mut()
        .map(|(company_id, bytes)| ManufacturerData {
            company_id: *company_id,
            company_id_hex: format!("0x{company_id:04X}"),
            data_hex: hex::encode_upper(bytes),
        })
        .collect()
}

fn service_data(data: std::collections::HashMap<Uuid, Vec<u8>>) -> Vec<ServiceData> {
    let mut entries = data
        .into_iter()
        .map(|(uuid, bytes)| ServiceData {
            service_uuid: uuid.to_string(),
            data_hex: hex::encode_upper(bytes),
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.service_uuid.cmp(&right.service_uuid));
    entries
}

fn sorted_uuids(mut uuids: Vec<Uuid>) -> Vec<Uuid> {
    uuids.sort();
    uuids
}

#[cfg(test)]
mod tests {
    #[test]
    fn hex_encoding_is_uppercase() {
        assert_eq!(hex::encode_upper([0x00, 0x2b, 0xff]), "002BFF");
    }
}
