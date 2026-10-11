use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use btleplug::{
    api::{Central, Peripheral as _, ScanFilter},
    platform::{Adapter, Peripheral},
};
use futures::StreamExt;
use serde_json::json;
use tokio::time::{sleep, timeout};

use crate::{
    ble::{gatt, scanner},
    diagnostics::recorder::{details_from_pairs, DiagnosticEvent, EventKind, JsonlRecorder},
    models::device::BleDeviceRecord,
    protocols,
};

pub async fn inspect(
    selector: &str,
    listen_seconds: u64,
    recorder: &mut JsonlRecorder,
) -> Result<()> {
    let adapter = scanner::first_adapter().await?;
    let adapter_name = adapter
        .adapter_info()
        .await
        .unwrap_or_else(|_| "<unbekannter Adapter>".to_string());
    recorder
        .write(&DiagnosticEvent::new(EventKind::AdapterDetected).with_adapter(&adapter_name))?;

    let peripheral = find_peripheral(&adapter, selector, &adapter_name, recorder)
        .await?
        .ok_or_else(|| {
            anyhow!("Geraet {selector:?} wurde nicht gefunden. Bitte vorher mit `scan` suchen.")
        })?;
    let device_id = peripheral.id().to_string();

    recorder.write(
        &DiagnosticEvent::new(EventKind::ConnectionStarted)
            .with_adapter(&adapter_name)
            .with_device(&device_id),
    )?;
    println!("Verbinde mit {device_id} ...");

    peripheral
        .connect_with_timeout(Duration::from_secs(20))
        .await
        .with_context(|| format!("Verbindung zu {device_id} fehlgeschlagen"))?;
    recorder.write(&DiagnosticEvent::new(EventKind::Connected).with_device(&device_id))?;

    let notification_stream = peripheral
        .notifications()
        .await
        .context("Notification-Stream konnte nicht angelegt werden")?;

    peripheral
        .discover_services_with_timeout(Duration::from_secs(20))
        .await
        .context("GATT-Service-Discovery fehlgeschlagen")?;

    let services = peripheral.services().into_iter().collect::<Vec<_>>();
    let service_uuids = gatt::service_uuids(&services);
    let characteristic_uuids = gatt::characteristic_uuids(&services);
    let protocol_matches = protocols::detect_from_gatt(&service_uuids, &characteristic_uuids);

    for profile in &protocol_matches {
        println!(
            "Profil-Kandidat: {} ({}) - {}",
            profile.protocol,
            profile.confidence,
            profile.evidence.join(", ")
        );
    }

    for service in &services {
        recorder.write(
            &DiagnosticEvent::new(EventKind::ServiceDiscovered)
                .with_device(&device_id)
                .with_service(service.uuid.to_string())
                .with_details(json!({
                    "primary": service.primary,
                    "profile_matches": protocol_matches,
                })),
        )?;
        println!("Service: {} (primary: {})", service.uuid, service.primary);

        for characteristic in &service.characteristics {
            let info = gatt::characteristic_info(characteristic);
            recorder.write(
                &DiagnosticEvent::new(EventKind::CharacteristicDiscovered)
                    .with_device(&device_id)
                    .with_service(&info.service_uuid)
                    .with_characteristic(&info.characteristic_uuid)
                    .with_details(json!({ "properties": info.properties })),
            )?;
            println!(
                "  Characteristic: {} [{}]",
                characteristic.uuid,
                gatt::property_names(characteristic.properties).join(", ")
            );

            if gatt::can_read(characteristic) {
                match timeout(Duration::from_secs(10), peripheral.read(characteristic)).await {
                    Ok(Ok(bytes)) => {
                        recorder.write(
                            &DiagnosticEvent::new(EventKind::CharacteristicRead)
                                .with_device(&device_id)
                                .with_service(characteristic.service_uuid.to_string())
                                .with_characteristic(characteristic.uuid.to_string())
                                .with_raw(&bytes)
                                .with_status("read_ok"),
                        )?;
                    }
                    Ok(Err(error)) => {
                        recorder.write(
                            &DiagnosticEvent::new(EventKind::Error)
                                .with_device(&device_id)
                                .with_service(characteristic.service_uuid.to_string())
                                .with_characteristic(characteristic.uuid.to_string())
                                .with_error(format!("Lesen fehlgeschlagen: {error}")),
                        )?;
                    }
                    Err(_) => {
                        recorder.write(
                            &DiagnosticEvent::new(EventKind::Error)
                                .with_device(&device_id)
                                .with_service(characteristic.service_uuid.to_string())
                                .with_characteristic(characteristic.uuid.to_string())
                                .with_error("Lesen nach 10 Sekunden abgebrochen"),
                        )?;
                    }
                }
            }

            if gatt::can_subscribe(characteristic) {
                match peripheral.subscribe(characteristic).await {
                    Ok(()) => {
                        recorder.write(
                            &DiagnosticEvent::new(EventKind::CharacteristicDiscovered)
                                .with_device(&device_id)
                                .with_service(characteristic.service_uuid.to_string())
                                .with_characteristic(characteristic.uuid.to_string())
                                .with_status("subscribed"),
                        )?;
                    }
                    Err(error) => {
                        recorder.write(
                            &DiagnosticEvent::new(EventKind::Error)
                                .with_device(&device_id)
                                .with_service(characteristic.service_uuid.to_string())
                                .with_characteristic(characteristic.uuid.to_string())
                                .with_error(format!("Subscription fehlgeschlagen: {error}")),
                        )?;
                    }
                }
            }
        }
    }

    println!("Warte {listen_seconds} Sekunden auf Notifications/Indications ...");
    listen_for_notifications(notification_stream, &device_id, listen_seconds, recorder).await?;

    for service in &services {
        for characteristic in &service.characteristics {
            if gatt::can_subscribe(characteristic) {
                let _ = peripheral.unsubscribe(characteristic).await;
            }
        }
    }

    let disconnect_result = peripheral.disconnect().await;
    match disconnect_result {
        Ok(()) => recorder
            .write(&DiagnosticEvent::new(EventKind::Disconnected).with_device(&device_id))?,
        Err(error) => recorder.write(
            &DiagnosticEvent::new(EventKind::ConnectionLost)
                .with_device(&device_id)
                .with_error(error.to_string()),
        )?,
    }

    Ok(())
}

async fn listen_for_notifications(
    mut notification_stream: impl futures::Stream<Item = btleplug::api::ValueNotification> + Unpin,
    device_id: &str,
    listen_seconds: u64,
    recorder: &mut JsonlRecorder,
) -> Result<()> {
    let deadline = sleep(Duration::from_secs(listen_seconds));
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            _ = &mut deadline => break,
            notification = notification_stream.next() => {
                match notification {
                    Some(value) => {
                        recorder.write(
                            &DiagnosticEvent::new(EventKind::NotificationReceived)
                                .with_device(device_id)
                                .with_characteristic(value.uuid.to_string())
                                .with_raw(&value.value)
                                .with_details(details_from_pairs(&[("payload_len", value.value.len().to_string())])),
                        )?;
                    }
                    None => {
                        recorder.write(
                            &DiagnosticEvent::new(EventKind::ConnectionLost)
                                .with_device(device_id)
                                .with_status("Notification-Stream beendet"),
                        )?;
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn find_peripheral(
    adapter: &Adapter,
    selector: &str,
    adapter_name: &str,
    recorder: &mut JsonlRecorder,
) -> Result<Option<Peripheral>> {
    if let Some(peripheral) = find_in_known_peripherals(adapter, selector, None).await? {
        return Ok(Some(peripheral));
    }

    recorder.write(
        &DiagnosticEvent::new(EventKind::ScanStarted)
            .with_adapter(adapter_name)
            .with_status("Geraetesuche fuer Inspect gestartet"),
    )?;
    adapter
        .start_scan(ScanFilter::default())
        .await
        .context("BLE-Scan fuer Geraetesuche konnte nicht gestartet werden")?;
    sleep(Duration::from_secs(6)).await;
    let _ = adapter.stop_scan().await;
    recorder.write(
        &DiagnosticEvent::new(EventKind::ScanCompleted)
            .with_adapter(adapter_name)
            .with_status("Geraetesuche fuer Inspect beendet"),
    )?;

    find_in_known_peripherals(adapter, selector, Some(recorder)).await
}

async fn find_in_known_peripherals(
    adapter: &Adapter,
    selector: &str,
    mut recorder: Option<&mut JsonlRecorder>,
) -> Result<Option<Peripheral>> {
    let peripherals = adapter.peripherals().await?;
    for peripheral in peripherals {
        if peripheral.id().to_string() == selector {
            return Ok(Some(peripheral));
        }

        if let Ok(Some(properties)) = peripheral.properties().await {
            let id = peripheral.id().to_string();
            let name = properties
                .local_name
                .clone()
                .or_else(|| properties.advertisement_name.clone());
            let address = properties.address.to_string();
            if let Some(writer) = recorder.as_deref_mut() {
                writer.write(
                    &DiagnosticEvent::new(EventKind::DeviceDiscovered)
                        .with_device(&id)
                        .with_details(json!({
                            "name": name,
                            "address": address,
                            "rssi": properties.rssi,
                        })),
                )?;
            }

            let record = BleDeviceRecord {
                id,
                name,
                address: Some(address),
                address_type: properties
                    .address_type
                    .map(|address_type| format!("{address_type:?}")),
                rssi: properties.rssi,
                tx_power_level: properties.tx_power_level,
                manufacturer_data: Vec::new(),
                advertised_services: Vec::new(),
                service_data: Vec::new(),
                possible_profiles: Vec::new(),
                last_seen_utc: chrono::Utc::now(),
            };
            if record.matches_user_selector(selector) {
                return Ok(Some(peripheral));
            }
        }
    }

    Ok(None)
}
