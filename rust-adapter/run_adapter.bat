@echo off
setlocal

if "%~1"=="" goto usage

set "BACKEND_URL=%~1"
if "%~2"=="" (
  set "KEYENCE_HOST=127.0.0.1"
) else (
  set "KEYENCE_HOST=%~2"
)

if "%~3"=="" (
  set "KEYENCE_PORT=9004"
) else (
  set "KEYENCE_PORT=%~3"
)

if "%~4"=="" (
  set "ADAPTER_EXE=access-control-system.exe"
) else (
  set "ADAPTER_EXE=%~4"
)

set "MACHINE_ID=MACHINE_1"
set "ADAPTER_TOKEN=done"
set "OVERRIDE_TOKEN=override-token"
set "OVERRIDE_PASSCODE=1234"
set "SERVER_HOST=0.0.0.0"
set "SERVER_PORT=8080"
set "PLC_PORT=COM3"
set "PLC_BAUDRATE=9600"
set "KEYENCE_PORT=%KEYENCE_PORT%"
set "KEYENCE_HOST=%KEYENCE_HOST%"
set "RUN_EMBEDDED_PYTHON=false"

echo Starting adapter with:
echo BACKEND_URL=%BACKEND_URL%
echo KEYENCE_HOST=%KEYENCE_HOST%
echo KEYENCE_PORT=%KEYENCE_PORT%
echo PLC_PORT=%PLC_PORT%
echo SERVER_PORT=%SERVER_PORT%

if not exist "%ADAPTER_EXE%" (
  echo ERROR: executable not found: %ADAPTER_EXE%
  echo Put this .bat in same folder as access-control-system.exe or pass full exe path as 4th argument.
  exit /b 1
)

"%ADAPTER_EXE%"
exit /b %ERRORLEVEL%

:usage
echo Usage:
echo run_adapter.bat http://^<BACKEND_IP^>:8000 [KEYENCE_HOST] [KEYENCE_PORT] [EXE_PATH]
echo Example:
echo run_adapter.bat http://192.168.1.10:8000 192.168.1.50 9004
exit /b 1
