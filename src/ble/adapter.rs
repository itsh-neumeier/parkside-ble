use anyhow::{Context, Result};
use btleplug::{
    api::{Central, Manager as _},
    platform::Manager,
};

use crate::models::device::AdapterInfo;

pub async fn list_adapters() -> Result<Vec<AdapterInfo>> {
    let manager = Manager::new()
        .await
        .context("Bluetooth-Manager konnte nicht initialisiert werden")?;
    let adapters = manager
        .adapters()
        .await
        .context("Bluetooth-Adapter konnten nicht gelesen werden")?;

    let mut infos = Vec::new();
    for adapter in adapters {
        let name = adapter
            .adapter_info()
            .await
            .unwrap_or_else(|_| "<unbekannter Adapter>".to_string());
        let address = adapter
            .adapter_address()
            .await
            .ok()
            .flatten()
            .map(|address| address.to_string());
        infos.push(AdapterInfo { name, address });
    }

    Ok(infos)
}
