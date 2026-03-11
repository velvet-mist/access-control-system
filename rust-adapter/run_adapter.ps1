param(
    [string]$BackendUrl = "http://127.0.0.1:8000",
    [string]$MachineId = "MACHINE_1",
    [string]$AdapterToken = "done",
    [string]$OverrideToken = "override-token",
    [string]$OverridePasscode = "1234",
    [string]$ServerHost = "0.0.0.0",
    [int]$ServerPort = 8080,
    [string]$KeyenceHost = "127.0.0.1",
    [int]$KeyencePort = 9004,
    [string]$PlcPort = "COM3",
    [int]$PlcBaudrate = 9600,
    [switch]$UseRelease
)

$ErrorActionPreference = "Stop"

$env:BACKEND_URL = $BackendUrl
$env:MACHINE_ID = $MachineId
$env:ADAPTER_TOKEN = $AdapterToken
$env:OVERRIDE_TOKEN = $OverrideToken
$env:OVERRIDE_PASSCODE = $OverridePasscode
$env:SERVER_HOST = $ServerHost
$env:SERVER_PORT = "$ServerPort"
$env:KEYENCE_HOST = $KeyenceHost
$env:KEYENCE_PORT = "$KeyencePort"
$env:PLC_PORT = $PlcPort
$env:PLC_BAUDRATE = "$PlcBaudrate"
$env:RUN_EMBEDDED_PYTHON = "false"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

Write-Host "Starting adapter with:"
Write-Host "BACKEND_URL=$env:BACKEND_URL"
Write-Host "KEYENCE_HOST=$env:KEYENCE_HOST"
Write-Host "KEYENCE_PORT=$env:KEYENCE_PORT"
Write-Host "PLC_PORT=$env:PLC_PORT"
Write-Host "PLC_BAUDRATE=$env:PLC_BAUDRATE"
Write-Host "SERVER_PORT=$env:SERVER_PORT"

if ($UseRelease) {
    cargo run --release
} else {
    cargo run
}
