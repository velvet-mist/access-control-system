param(
    [string]$BackendUrl = "",
    [string]$MachineId = "",
    [string]$AdapterToken = "",
    [string]$OverrideToken = "",
    [string]$OverridePasscode = "",
    [string]$ServerHost = "",
    [string]$ServerPort = "",
    [string]$KeyenceHost = "",
    [string]$KeyencePort = "",
    [string]$PlcPort = "",
    [string]$PlcBaudrate = "",
    [string]$PlcHost = "",
    [string]$PlcTcpPort = "",
    [string]$PlcSlaveAddr = "",
    [switch]$UseRelease
)

$ErrorActionPreference = "Stop"

if ($BackendUrl) { $env:BACKEND_URL = $BackendUrl }
if ($MachineId) { $env:MACHINE_ID = $MachineId }
if ($AdapterToken) { $env:ADAPTER_TOKEN = $AdapterToken }
if ($OverrideToken) { $env:OVERRIDE_TOKEN = $OverrideToken }
if ($OverridePasscode) { $env:OVERRIDE_PASSCODE = $OverridePasscode }
if ($ServerHost) { $env:SERVER_HOST = $ServerHost }
if ($ServerPort) { $env:SERVER_PORT = "$ServerPort" }
if ($KeyenceHost) { $env:KEYENCE_HOST = $KeyenceHost }
if ($KeyencePort) { $env:KEYENCE_PORT = "$KeyencePort" }
if ($PlcPort) { $env:PLC_PORT = $PlcPort }
if ($PlcBaudrate) { $env:PLC_BAUDRATE = "$PlcBaudrate" }
if ($PlcHost) { $env:PLC_HOST = $PlcHost }
if ($PlcTcpPort) { $env:PLC_TCP_PORT = "$PlcTcpPort" }
if ($PlcSlaveAddr) { $env:PLC_SLAVE_ADDR = "$PlcSlaveAddr" }
$env:RUN_EMBEDDED_PYTHON = "false"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

Write-Host "Starting adapter with:"
Write-Host "BACKEND_URL=$env:BACKEND_URL"
Write-Host "KEYENCE_HOST=$env:KEYENCE_HOST"
Write-Host "KEYENCE_PORT=$env:KEYENCE_PORT"
Write-Host "PLC_HOST=$env:PLC_HOST"
Write-Host "PLC_TCP_PORT=$env:PLC_TCP_PORT"
Write-Host "PLC_PORT=$env:PLC_PORT"
Write-Host "PLC_BAUDRATE=$env:PLC_BAUDRATE"
Write-Host "SERVER_PORT=$env:SERVER_PORT"

if ($UseRelease) {
    cargo run --release
} else {
    cargo run
}
