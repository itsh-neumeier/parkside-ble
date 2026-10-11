# PARKSIDE Smart Battery Manager

Diagnoseprogramm fuer PARKSIDE Smart 20-V-Akkus mit 4 Ah, 8 Ah und 12 Ah ueber Bluetooth Low Energy. Der aktuelle Stand ist ein lesendes CLI-Werkzeug: Es scannt BLE-Geraete, erkennt moegliche PARKSIDE-/Tuya-BLE-Merkmale, untersucht GATT-Services und speichert Rohdaten als JSONL.

## Unterstuetzte Systeme

- Windows 10/11 x64
- Linux x86_64 mit BlueZ/D-Bus

Die BLE-Kommunikation nutzt `btleplug`. Unter Windows und Linux koennen sich Geraete-IDs und Bluetooth-Adressen unterscheiden; fuer die Auswahl wird die portable `btleplug`-Geraete-ID bevorzugt.

## Voraussetzungen

Fuer fertige GitHub-Artefakte ist keine lokale Rust-Installation noetig.

Fuer lokale Builds:

- Rust Stable mit Cargo
- Windows: Microsoft C++ Build Tools / MSVC-Linker `link.exe`
- Linux: BlueZ-/D-Bus-Entwicklungsbibliotheken, zum Beispiel `pkg-config libdbus-1-dev libudev-dev bluez`

## Windows-Schnellstart

1. GitHub-Artefakt `parkside-ble-diagnostics-windows-x64.zip` herunterladen.
2. ZIP entpacken.
3. `Start-Parkside.bat` starten.
4. Im Menue zuerst Adapter anzeigen, dann scannen, danach die GATT-Diagnose fuer die angezeigte Geraete-ID starten.

Direkt per PowerShell:

```powershell
.\parkside-ble-diagnostics.exe adapters
.\parkside-ble-diagnostics.exe scan --seconds 20
.\parkside-ble-diagnostics.exe inspect --device "GERAETE-ID" --listen-seconds 60 --output "logs\akku-01.jsonl"
```

## Linux-Schnellstart

```bash
sudo apt-get install pkg-config libdbus-1-dev libudev-dev bluez
cargo build --release --locked
./target/release/parkside-ble-diagnostics adapters
./target/release/parkside-ble-diagnostics scan --seconds 20
./target/release/parkside-ble-diagnostics inspect --device "GERAETE-ID" --listen-seconds 60 --output logs/akku-01.jsonl
```

Je nach Distribution sind Bluetooth-Rechte oder ein laufender BlueZ-Dienst erforderlich.

## BLE-Scan

```powershell
parkside-ble-diagnostics scan --seconds 20
parkside-ble-diagnostics scan --seconds 20 --json
parkside-ble-diagnostics scan --all
```

Der Scanner speichert und zeigt alle erkannten BLE-Geraete, auch ohne Namen. Ausgegeben werden, soweit vom Betriebssystem geliefert:

- Name
- portable Geraete-ID
- Bluetooth-Adresse
- RSSI
- Manufacturer Data
- Advertised Services
- Service Data
- Zeitpunkt der letzten Erkennung
- moegliche Protokollkandidaten

## Geraeteauswahl

Nutze bevorzugt die `ID` aus `scan`. Wenn noetig, kann die Diagnose auch exakt passende Namen oder Adressen erkennen. IDs mit Leerzeichen oder Sonderzeichen muessen in Anfuehrungszeichen stehen.

## GATT-Diagnose

```powershell
parkside-ble-diagnostics inspect --device "GERAETE-ID" --listen-seconds 60
parkside-ble-diagnostics inspect --device "GERAETE-ID" --output "logs\akku-01.jsonl"
```

Der Inspect-Modus:

- sucht das Geraet per BLE-Scan,
- verbindet sich,
- fuehrt Service Discovery aus,
- listet Services und Characteristics,
- liest lesbare Characteristics,
- abonniert Notify-/Indicate-Characteristics,
- protokolliert eingehende Rohdaten mit UTC-Zeitstempel,
- trennt die Verbindung sauber.

Es werden keine unbekannten Schreibbefehle, Firmware-Aenderungen oder experimentellen Tuya-Kommandos gesendet.

## JSONL-Protokolle

Standarddatei:

```text
parkside-diagnostics.jsonl
```

Jede Zeile ist genau ein JSON-Objekt. Typische Felder:

- `timestamp_utc`
- `event_type`
- `adapter`
- `device_id`
- `service_uuid`
- `characteristic_uuid`
- `raw_hex`
- `interpretation`
- `status`
- `error`
- `details`

Ereignistypen sind unter anderem `adapter_detected`, `scan_started`, `device_discovered`, `connected`, `service_discovered`, `characteristic_read`, `notification_received`, `connection_lost`, `disconnected` und `error`.

Bericht erzeugen:

```powershell
parkside-ble-diagnostics report --input "logs\akku-01.jsonl"
```

## PARKSIDE- und Tuya-Hinweise

Bekannte oeffentliche Hinweise auf PARKSIDE-Smart-Akkus zeigen moegliche Tuya-BLE-Merkmale:

- GATT-Service `0x1910`
- Characteristic `0x2B10`
- Characteristic `0x2B11`

Diese UUIDs sind Kandidaten, keine Garantie fuer alle Modelle. Verschluesselte oder unbekannte Payloads werden nur als Hex-Rohdaten gespeichert. Das Programm leitet keine erfundenen Akkustand-, Temperatur-, Spannungs- oder Zykluswerte ab.

## GitHub-Actions-Build

Workflow: `.github/workflows/build.yml`

Er laeuft manuell und bei Pushes auf `main` oder `master`:

- Rust Stable installieren
- Linux-BLE-Abhaengigkeiten installieren
- `cargo fetch --locked`
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test`
- `cargo build --release --locked`
- Artefakte hochladen

Artefakte unter **Actions -> Build PARKSIDE BLE Diagnostics -> Artifacts**:

- `parkside-ble-diagnostics-windows-x64.zip`
- `parkside-ble-diagnostics-linux-x64.tar.gz`

## Typische Fehler und Loesungen

- Kein Adapter gefunden: Bluetooth aktivieren, USB-Adapter pruefen, Treiber installieren.
- Scan startet nicht: Unter Linux BlueZ-Dienst und Berechtigungen pruefen.
- Windows-Build bricht bei `link.exe` ab: Microsoft C++ Build Tools installieren.
- Geraet nicht gefunden: Akku wecken, naeher an den Rechner bringen, erneut `scan --seconds 20` ausfuehren.
- Characteristic-Lesen schlaegt fehl: Manche Characteristics sind nicht lesbar oder erfordern Authentifizierung; der Fehler wird im JSONL protokolliert.

## Bekannte Einschraenkungen

- BLE-Hardwaretests mit echten PARKSIDE-Akkus sind in CI nicht moeglich und muessen manuell erfolgen.
- Tuya-/PARKSIDE-Payloads koennen verschluesselt sein und werden derzeit nicht entschluesselt.
- Noch keine GUI, SQLite-Datenbank, MQTT-Integration oder Home-Assistant-Anbindung.
- Keine Steuerung von Akku-Modi oder BMS-Parametern.

## Geplante Erweiterungen

- Tauri-2-Desktopoberflaeche
- Verwaltung mehrerer Akkus
- SQLite-Messwerthistorie
- Messwertinterpretation, sobald das Protokoll sicher bekannt ist
- Diagramme und Langzeitauswertung
- Automatische Wiederverbindung
- CSV- und JSON-Export interpretierter Messwerte
- Optional MQTT und Home Assistant
- Sichere Verwaltung offizieller Modi, sobald Schreibbefehle verlaesslich bekannt sind

## Lokale Validierung

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release --locked
```

Ein erfolgreicher Build beweist nur, dass das Programm kompiliert und hardwareunabhaengige Tests besteht. Reale BLE-Kommunikation bleibt **nicht hardwaregetestet**, bis ein PARKSIDE Smart-Akku verbunden wurde.
