use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use anyhow::{Context, Result};

use crate::diagnostics::recorder::{DiagnosticEvent, EventKind};

#[derive(Debug, Default)]
pub struct DiagnosticReport {
    pub total_events: usize,
    pub event_counts: BTreeMap<String, usize>,
    pub devices: BTreeSet<String>,
    pub errors: Vec<String>,
}

impl DiagnosticReport {
    pub fn render_text(&self) -> String {
        let mut text = String::new();
        text.push_str("Diagnosebericht\n");
        text.push_str("================\n");
        text.push_str(&format!("Ereignisse gesamt: {}\n", self.total_events));
        text.push_str(&format!("Geraete: {}\n", self.devices.len()));

        if !self.event_counts.is_empty() {
            text.push_str("\nEreignistypen:\n");
            for (event, count) in &self.event_counts {
                text.push_str(&format!("- {event}: {count}\n"));
            }
        }

        if !self.errors.is_empty() {
            text.push_str("\nFehler:\n");
            for error in &self.errors {
                text.push_str(&format!("- {error}\n"));
            }
        }

        text
    }
}

pub fn read_report(path: &Path) -> Result<DiagnosticReport> {
    let file = File::open(path).with_context(|| format!("Kann {:?} nicht lesen", path))?;
    let reader = BufReader::new(file);
    let mut report = DiagnosticReport::default();

    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let event: DiagnosticEvent = serde_json::from_str(&line)
            .with_context(|| format!("Ungueltiges JSONL in Zeile {}", line_number + 1))?;
        report.total_events += 1;
        *report
            .event_counts
            .entry(event_name(event.event_type).to_string())
            .or_default() += 1;

        if let Some(device_id) = event.device_id {
            report.devices.insert(device_id);
        }
        if let Some(error) = event.error {
            report.errors.push(error);
        }
    }

    Ok(report)
}

fn event_name(event: EventKind) -> &'static str {
    match event {
        EventKind::AdapterDetected => "adapter_detected",
        EventKind::ScanStarted => "scan_started",
        EventKind::DeviceDiscovered => "device_discovered",
        EventKind::ScanCompleted => "scan_completed",
        EventKind::ConnectionStarted => "connection_started",
        EventKind::Connected => "connected",
        EventKind::ServiceDiscovered => "service_discovered",
        EventKind::CharacteristicDiscovered => "characteristic_discovered",
        EventKind::CharacteristicRead => "characteristic_read",
        EventKind::NotificationReceived => "notification_received",
        EventKind::ConnectionLost => "connection_lost",
        EventKind::Disconnected => "disconnected",
        EventKind::Error => "error",
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;
    use crate::diagnostics::recorder::DiagnosticEvent;

    #[test]
    fn report_counts_events_and_errors() {
        let mut file = NamedTempFile::new().expect("temp file");
        let event = DiagnosticEvent::new(EventKind::Error)
            .with_device("akku-1")
            .with_error("read failed");
        writeln!(file, "{}", serde_json::to_string(&event).expect("json")).expect("write");

        let report = read_report(file.path()).expect("report");
        assert_eq!(report.total_events, 1);
        assert_eq!(report.devices.len(), 1);
        assert_eq!(report.errors, vec!["read failed"]);
    }
}
