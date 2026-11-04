# Runs TextilLab from the repository root or any PowerShell working directory.
$ErrorActionPreference = 'Stop'
$ProjectDir = Join-Path $PSScriptRoot '..\textillab'
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "Rust/Cargo ist nicht installiert. Installiere Rust von https://rustup.rs und starte PowerShell neu."
    exit 1
}
Push-Location $ProjectDir
try {
    cargo test
    if ($LASTEXITCODE -ne 0) { throw "Unit-Tests fehlgeschlagen" }
    cargo run --release --bin textillab-demo -- exports
    if ($LASTEXITCODE -ne 0) { throw "Export fehlgeschlagen" }
    $output = Join-Path (Get-Location) 'exports'
    Write-Host "Fertig. OBJ und SVG unter $output"
    if (Test-Path $output) { Start-Process explorer.exe $output }
} finally {
    Pop-Location
}
