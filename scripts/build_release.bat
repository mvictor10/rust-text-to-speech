@echo off
setlocal
cd /d "%~dp0.."

echo ==============================================
echo  Rust Voice Studio - compilacao Release
echo ==============================================

where cargo >nul 2>nul
if errorlevel 1 (
    echo ERRO: Cargo nao foi encontrado.
    echo Instale Rust pelo rustup e abra um novo terminal.
    pause
    exit /b 1
)

where rustc >nul 2>nul
if errorlevel 1 (
    echo ERRO: rustc nao foi encontrado.
    pause
    exit /b 1
)

for /f "tokens=2" %%V in ('rustc --version') do echo Rust: %%V
cargo check
if errorlevel 1 goto :failed

cargo build --release
if errorlevel 1 goto :failed

if not exist "dist" mkdir "dist"
copy /Y "target\release\rust_voice_studio.exe" "dist\RustVoiceStudio.exe" >nul
if errorlevel 1 goto :failed

echo.
echo Compilacao concluida.
echo Executavel: dist\RustVoiceStudio.exe
pause
exit /b 0

:failed
echo.
echo A compilacao falhou. Consulte as mensagens acima e o README.md.
pause
exit /b 1
