@echo off
setlocal

if "%~1"=="" goto usage
if "%~2"=="" goto usage

set "BACKEND_URL=%~1"
set "COGNEX_HOST=%~2"
if "%~3"=="" (
  set "COGNEX_PORT=23"
) else (
  set "COGNEX_PORT=%~3"
)

if "%~4"=="" (
  set "ADAPTER_EXE=access-control-system.exe"
) else (
  set "ADAPTER_EXE=%~4"
)

set "PLC_TYPE=cognex"
set "MACHINE_ID=MACHINE_1"
set "ADAPTER_TOKEN=done"
set "OVERRIDE_TOKEN=override-token"
set "OVERRIDE_PASSCODE=1234"
set "SERVER_HOST=0.0.0.0"
set "SERVER_PORT=8080"
set "COGNEX_ALLOW_COMMAND=ALLOW"
set "COGNEX_DENY_COMMAND=DENY"
set "COGNEX_RESET_COMMAND=RESET"
set "RUN_EMBEDDED_PYTHON=false"

echo Starting adapter with:
echo BACKEND_URL=%BACKEND_URL%
echo COGNEX_HOST=%COGNEX_HOST%
echo COGNEX_PORT=%COGNEX_PORT%
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
echo run_adapter.bat http://^<BACKEND_IP^>:8000 ^<COGNEX_IP^> [COGNEX_PORT] [EXE_PATH]
echo Example:
echo run_adapter.bat http://192.168.1.10:8000 192.168.1.50 23
exit /b 1
