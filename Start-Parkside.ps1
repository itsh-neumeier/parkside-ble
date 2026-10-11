Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$LogDir = Join-Path $Root "logs"
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null

function Find-ParksideExe {
    $candidates = @(
        (Join-Path $Root "parkside-ble-diagnostics.exe"),
        (Join-Path $Root "target\release\parkside-ble-diagnostics.exe")
    )

    foreach ($candidate in $candidates) {
        if (Test-Path -LiteralPath $candidate) {
            return $candidate
        }
    }
    return $null
}

function Ensure-ParksideExe {
    $exe = Find-ParksideExe
    if ($exe) {
        return $exe
    }

    Write-Host "Keine vorkompilierte EXE gefunden." -ForegroundColor Yellow
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if (-not $cargo) {
        Write-Host "Cargo ist nicht installiert. Lade ein Release-Artefakt von GitHub Actions herunter oder installiere Rust." -ForegroundColor Red
        return $null
    }

    $link = Get-Command link.exe -ErrorAction SilentlyContinue
    if (-not $link) {
        Write-Host "MSVC-Linker link.exe wurde nicht gefunden." -ForegroundColor Yellow
        Write-Host "Installiere die Microsoft C++ Build Tools mit der Komponente 'Desktop development with C++'."
        Write-Host "Falls Visual Studio installiert ist, starte diesen Starter aus einer Developer PowerShell."
    }

    $answer = Read-Host "Lokalen Release-Build mit Cargo starten? (j/N)"
    if ($answer -notmatch '^[jJyY]') {
        return $null
    }

    Push-Location $Root
    try {
        & cargo build --release --locked
        if ($LASTEXITCODE -ne 0) {
            throw "Cargo-Build fehlgeschlagen."
        }
    } finally {
        Pop-Location
    }

    return Find-ParksideExe
}

function New-LogPath {
    param([string]$Prefix)
    $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
    return Join-Path $LogDir "$Prefix-$stamp.jsonl"
}

function Invoke-Parkside {
    param(
        [Parameter(Mandatory=$true)][string]$Exe,
        [Parameter(ValueFromRemainingArguments=$true)][string[]]$Arguments
    )
    & $Exe @Arguments
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Befehl beendet mit Exitcode $LASTEXITCODE." -ForegroundColor Yellow
    }
}

while ($true) {
    Clear-Host
    Write-Host "PARKSIDE Smart Battery Manager - BLE Diagnostics"
    Write-Host "================================================"
    Write-Host "1) Bluetooth-Adapter anzeigen"
    Write-Host "2) BLE-Scan starten"
    Write-Host "3) GATT-Diagnose starten"
    Write-Host "4) Diagnosebericht anzeigen"
    Write-Host "5) Release-Build lokal erstellen"
    Write-Host "0) Beenden"
    Write-Host ""

    $choice = Read-Host "Auswahl"
    if ($choice -eq "0") {
        break
    }

    $exe = Ensure-ParksideExe
    if (-not $exe -and $choice -ne "5") {
        Read-Host "Weiter mit Enter"
        continue
    }

    try {
        switch ($choice) {
            "1" {
                Invoke-Parkside -Exe $exe -Arguments "adapters"
            }
            "2" {
                $seconds = Read-Host "Scan-Dauer in Sekunden (Standard 20)"
                if ([string]::IsNullOrWhiteSpace($seconds)) { $seconds = "20" }
                Invoke-Parkside -Exe $exe -Arguments "scan", "--seconds", $seconds
            }
            "3" {
                $device = Read-Host "Geraete-ID aus dem Scan"
                if ([string]::IsNullOrWhiteSpace($device)) {
                    Write-Host "Keine Geraete-ID eingegeben." -ForegroundColor Yellow
                    break
                }
                $seconds = Read-Host "Listen-Dauer in Sekunden (Standard 60)"
                if ([string]::IsNullOrWhiteSpace($seconds)) { $seconds = "60" }
                $output = New-LogPath -Prefix "inspect"
                Invoke-Parkside -Exe $exe -Arguments "inspect", "--device", $device, "--listen-seconds", $seconds, "--output", $output
                Write-Host "Logdatei: $output"
            }
            "4" {
                $input = Read-Host "Pfad zur JSONL-Datei"
                if ([string]::IsNullOrWhiteSpace($input)) {
                    Write-Host "Keine Datei eingegeben." -ForegroundColor Yellow
                    break
                }
                Invoke-Parkside -Exe $exe -Arguments "report", "--input", $input
            }
            "5" {
                Push-Location $Root
                try {
                    & cargo build --release --locked
                } finally {
                    Pop-Location
                }
            }
            default {
                Write-Host "Unbekannte Auswahl." -ForegroundColor Yellow
            }
        }
    } catch {
        Write-Host "Fehler: $($_.Exception.Message)" -ForegroundColor Red
    }

    Write-Host ""
    Read-Host "Weiter mit Enter"
}
