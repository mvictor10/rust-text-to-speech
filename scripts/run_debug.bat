@echo off
setlocal
cd /d "%~dp0.."
where cargo >nul 2>nul
if errorlevel 1 (
    echo Cargo nao foi encontrado. Instale Rust pelo rustup.
    pause
    exit /b 1
)
cargo run
if errorlevel 1 pause
