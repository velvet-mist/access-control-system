param(
    [switch]$SkipZip
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir

Write-Host "Building Windows release binary..."
cargo build --release

$distDir = Join-Path $scriptDir "dist\windows"
New-Item -ItemType Directory -Path $distDir -Force | Out-Null

$exePath = Join-Path $scriptDir "target\release\access-control-system.exe"
if (!(Test-Path $exePath)) {
    throw "Build finished but executable not found at $exePath"
}

Copy-Item -Force $exePath (Join-Path $distDir "access-control-system.exe")
Copy-Item -Force (Join-Path $scriptDir "run_adapter.bat") (Join-Path $distDir "run_adapter.bat")

$readme = @"
Windows deployment package

Files:
- access-control-system.exe
- run_adapter.bat

Run:
run_adapter.bat http://<BACKEND_IP>:8000 <COGNEX_IP> 23
"@

$readmePath = Join-Path $distDir "README.txt"
Set-Content -Path $readmePath -Value $readme -Encoding ascii

if (-not $SkipZip) {
    $zipPath = Join-Path $distDir "adapter-windows.zip"
    if (Test-Path $zipPath) {
        Remove-Item -Force $zipPath
    }
    Compress-Archive -Path (Join-Path $distDir "*") -DestinationPath $zipPath
    Write-Host "Created zip package: $zipPath"
}

Write-Host "Build output ready in: $distDir"
