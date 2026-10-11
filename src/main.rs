mod ble;
mod cli;
mod diagnostics;
mod models;
mod protocols;

use anyhow::Context;
use clap::Parser;
use cli::{Cli, Commands};
use diagnostics::recorder::{DiagnosticEvent, EventKind, JsonlRecorder};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    init_tracing();

    if let Err(error) = run().await {
        eprintln!("Fehler: {error:#}");
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Adapters => {
            let adapters = ble::adapter::list_adapters().await?;
            if adapters.is_empty() {
                println!("Keine Bluetooth-Adapter gefunden.");
                return Ok(());
            }

            for adapter in adapters {
                println!(
                    "- {}{}",
                    adapter.name,
                    adapter
                        .address
                        .as_deref()
                        .map(|address| format!(" ({address})"))
                        .unwrap_or_default()
                );
            }
        }
        Commands::Scan { seconds, json, all } => {
            let devices = ble::scanner::scan(seconds, all).await?;
            if json {
                println!("{}", serde_json::to_string_pretty(&devices)?);
            } else if devices.is_empty() {
                println!("Keine BLE-Geraete im Scanfenster erkannt.");
            } else {
                println!("{} BLE-Geraet(e) erkannt:", devices.len());
                for device in devices {
                    println!(
                        "- ID: {} | Name: {} | Adresse: {} | RSSI: {} | Profile: {}",
                        device.id,
                        device.name.as_deref().unwrap_or("<ohne Name>"),
                        device.address.as_deref().unwrap_or("<nicht verfuegbar>"),
                        device
                            .rssi
                            .map(|rssi| format!("{rssi} dBm"))
                            .unwrap_or_else(|| "<nicht verfuegbar>".to_string()),
                        device.profile_summary()
                    );
                }
            }
        }
        Commands::Inspect {
            device,
            listen_seconds,
            output,
        } => {
            let mut recorder = JsonlRecorder::create(&output)
                .with_context(|| format!("Kann {:?} nicht oeffnen", output))?;
            recorder
                .write(&DiagnosticEvent::new(EventKind::ConnectionStarted).with_device(&device))?;

            println!("Suche Geraet: {device}");
            println!("JSONL-Protokoll: {}", output.display());

            ble::connection::inspect(&device, listen_seconds, &mut recorder).await?;
            recorder.flush()?;
            println!("Diagnose abgeschlossen.");
        }
        Commands::Report { input } => {
            let report = diagnostics::report::read_report(&input)?;
            println!("{}", report.render_text());
        }
    }

    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));
    let _ = fmt().with_env_filter(filter).with_target(false).try_init();
}
