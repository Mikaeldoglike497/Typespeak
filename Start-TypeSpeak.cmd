@echo off
setlocal
cd /d "%~dp0"

if exist "src-tauri\target\debug\typespeak.exe" (
  start "" "src-tauri\target\debug\typespeak.exe"
  exit /b 0
)

where cargo >nul 2>nul
if errorlevel 1 (
  echo TypeSpeak is not built and Rust Cargo was not found.
  echo Install the Rust toolchain, then run:
  echo cargo build --manifest-path .\src-tauri\Cargo.toml
  pause
  exit /b 1
)

cargo run --manifest-path ".\src-tauri\Cargo.toml"
