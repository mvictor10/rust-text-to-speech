@echo off
setlocal
cd /d "%~dp0.."
cargo clean
if errorlevel 1 pause
