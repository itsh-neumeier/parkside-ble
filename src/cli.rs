use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "PARKSIDE Smart Battery Manager - BLE Diagnostics"
)]
#[command(
    long_about = "Diagnose-CLI fuer BLE-Scanning, GATT-Inspection und JSONL-Protokolle von PARKSIDE Smart-Akkus und anderen BLE-Geraeten."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Verfuegbare Bluetooth-Adapter anzeigen.
    Adapters,
    /// BLE-Geraete suchen.
    Scan {
        /// Dauer des Scans in Sekunden.
        #[arg(long, default_value_t = 20)]
        seconds: u64,
        /// Ergebnis als JSON ausgeben.
        #[arg(long)]
        json: bool,
        /// Alle Geraete anzeigen; Diagnose filtert ohnehin keine unbekannten Geraete aus.
        #[arg(long)]
        all: bool,
    },
    /// GATT-Services und Characteristics eines Geraets untersuchen.
    Inspect {
        /// Geraete-ID aus dem Scan. Leerzeichen und Sonderzeichen sind erlaubt, bitte quoten.
        #[arg(long)]
        device: String,
        /// Dauer fuer Notification-/Indication-Listening in Sekunden.
        #[arg(long, default_value_t = 60)]
        listen_seconds: u64,
        /// JSONL-Ausgabedatei.
        #[arg(long, default_value = "parkside-diagnostics.jsonl")]
        output: PathBuf,
    },
    /// Diagnoseprotokoll zusammenfassen.
    Report {
        /// JSONL-Datei aus einem Inspect-Lauf.
        #[arg(long)]
        input: PathBuf,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_accepts_device_ids_with_spaces() {
        let cli = Cli::try_parse_from([
            "parkside-ble-diagnostics",
            "inspect",
            "--device",
            "Windows BLE Device {ABC DEF}",
            "--listen-seconds",
            "5",
        ])
        .expect("CLI parse");

        match cli.command {
            Commands::Inspect {
                device,
                listen_seconds,
                ..
            } => {
                assert_eq!(device, "Windows BLE Device {ABC DEF}");
                assert_eq!(listen_seconds, 5);
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn clap_help_is_valid() {
        Cli::command().debug_assert();
    }
}
