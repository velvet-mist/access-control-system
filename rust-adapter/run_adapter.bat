@echo off
setlocal

if "%~1"=="" goto usage

set "BACKEND_URL=%~1"
if not "%~2"=="" (
  set "KEYENCE_HOST=%~2"
)

if not "%~3"=="" (
  set "KEYENCE_PORT=%~3"
)

if "%~4"=="" (
  set "ADAPTER_EXE=access-control-system.exe"
) else (
  set "ADAPTER_EXE=%~4"
)

set "RUN_EMBEDDED_PYTHON=false"

echo Starting adapter with:
echo BACKEND_URL=%BACKEND_URL%
echo KEYENCE_HOST=%KEYENCE_HOST%
echo KEYENCE_PORT=%KEYENCE_PORT%
echo PLC_HOST=%PLC_HOST%
echo PLC_TCP_PORT=%PLC_TCP_PORT%
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
