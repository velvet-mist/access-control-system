param(
    [string]$BackendUrl = "http://127.0.0.1:8000",
    [string]$MachineId = "MACHINE_1",
    [string]$AdapterToken = "done",
    [string]$OverrideToken = "override-token",
    [string]$OverridePasscode = "1234",
    [string]$ServerHost = "0.0.0.0",
    [int]$ServerPort = 8080,
    [string]$CognexHost = "127.0.0.1",
    [int]$CognexPort = 23,
    [string]$CognexAllowCommand = "ALLOW",
    [string]$CognexDenyCommand = "DENY",
    [string]$CognexResetCommand = "RESET",
    [switch]$UseRelease
)

$ErrorActionPreference = "Stop"

# Always run Cognex profile on this Windows launcher.
$env:PLC_TYPE = "cognex"
$env:BACKEND_URL = $BackendUrl
$env:MACHINE_ID = $MachineId
$env:ADAPTER_TOKEN = $AdapterToken
$env:OVERRIDE_TOKEN = $OverrideToken
$env:OVERRIDE_PASSCODE = $OverridePasscode
$env:SERVER_HOST = $ServerHost
$env:SERVER_PORT = "$ServerPort"
$env:COGNEX_HOST = $CognexHost
$env:COGNEX_PORT = "$CognexPort"
$env:COGNEX_ALLOW_COMMAND = $CognexAllowCommand
$env:COGNEX_DENY_COMMAND = $CognexDenyCommand
$env:COGNEX_RESET_COMMAND = $CognexResetCommand
$env:RUN_EMBEDDED_PYTHON = "false"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

Write-Host "Starting adapter with:"
Write-Host "BACKEND_URL=$env:BACKEND_URL"
Write-Host "PLC_TYPE=$env:PLC_TYPE"
Write-Host "COGNEX_HOST=$env:COGNEX_HOST"
Write-Host "COGNEX_PORT=$env:COGNEX_PORT"
Write-Host "SERVER_PORT=$env:SERVER_PORT"

if ($UseRelease) {
    cargo run --release
} else {
    cargo run
}
