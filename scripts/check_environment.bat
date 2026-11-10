@echo off
setlocal
cd /d "%~dp0.."
echo Verificando ambiente de desenvolvimento...
echo.
where rustup >nul 2>nul
if errorlevel 1 echo [AVISO] rustup nao encontrado.
where rustc >nul 2>nul
if errorlevel 1 (echo [AVISO] rustc nao encontrado.) else rustc --version
where cargo >nul 2>nul
if errorlevel 1 (echo [AVISO] cargo nao encontrado.) else cargo --version
echo.
echo Para compilar, use a toolchain x86_64-pc-windows-msvc e Visual Studio Build Tools com C++.
echo Consulte README.md.
pause
