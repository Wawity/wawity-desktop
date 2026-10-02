@echo off
setlocal
chcp 65001 >nul
title Wawity Build
cd /d "%~dp0"

set DIST=dist-build
call :prep_cargo

:menu
echo(
echo   ================ WAWITY BUILD ================
echo(
echo    [1] DESKTOP  —  фронт + Rust + установщик  →  %DIST%\WawitySetup-Desktop.exe
echo    [2] CLI      —  бинарник + установщик       →  %DIST%\WawitySetup-CLI.exe
echo    [3] Оба продукта подряд
echo(
echo    [4] Очистка сборки
echo    [5] Диагностика cargo: реестр и сеть
echo    [0] Выход
echo(
set PICK=
set /p PICK=   Выбор: 
if "%PICK%"=="1" goto desktop
if "%PICK%"=="2" goto cli
if "%PICK%"=="3" goto both
if "%PICK%"=="4" goto clean
if "%PICK%"=="5" goto diag
if "%PICK%"=="0" exit /b 0
goto menu

:desktop
call :product_desktop || goto fail
pause
goto menu

:cli
call :product_cli || goto fail
pause
goto menu

:both
call :product_desktop || goto fail
call :product_cli || goto fail
pause
goto menu

:clean
cargo clean
pushd installer
cargo clean
popd
echo   Очищено.
pause
goto menu

:fail
echo(
echo   [X] Сборка прервана из-за ошибки.
pause
goto menu

:prep_cargo
set "MIRROR=0"
set "CLEAN_HOME=%~dp0.cargo-home"
set "CFG_USER=%USERPROFILE%\.cargo\config.toml"
set "CFG_USER2=%USERPROFILE%\.cargo\config"
set "CFG_HOME="
set "CFG_HOME2="
if defined CARGO_HOME set "CFG_HOME=%CARGO_HOME%\config.toml"
if defined CARGO_HOME set "CFG_HOME2=%CARGO_HOME%\config"
if exist "%CFG_USER%" call :scan_cfg "%CFG_USER%"
if exist "%CFG_USER2%" call :scan_cfg "%CFG_USER2%"
if defined CFG_HOME if exist "%CFG_HOME%" call :scan_cfg "%CFG_HOME%"
if defined CFG_HOME2 if exist "%CFG_HOME2%" call :scan_cfg "%CFG_HOME2%"
if defined CARGO_SOURCE_CRATES_IO_REPLACE_WITH set "MIRROR=1"
if "%MIRROR%"=="0" exit /b 0
echo(
echo   [i] Обнаружено зеркало cargo в глобальном конфиге — параметр replace-with.
echo       Сборка пойдёт через изолированный CARGO_HOME:
echo       %CLEAN_HOME%
if not exist "%CLEAN_HOME%" mkdir "%CLEAN_HOME%"
if exist "%CLEAN_HOME%\config.toml" del /q "%CLEAN_HOME%\config.toml"
if exist "%CLEAN_HOME%\config" del /q "%CLEAN_HOME%\config"
set "CARGO_HOME=%CLEAN_HOME%"
set "CARGO_SOURCE_CRATES_IO_REPLACE_WITH="
set "CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse"
set "CARGO_NET_RETRY=3"
exit /b 0

:scan_cfg
findstr /i /c:"replace-with" %1 >nul 2>&1
if not errorlevel 1 set "MIRROR=1"
exit /b 0

:diag
echo(
echo   --- ДИАГНОСТИКА CARGO ---
echo(
where cargo
cargo --version
echo(
echo   CARGO_HOME для сборки: %CARGO_HOME%
echo   Зеркало обнаружено: %MIRROR%
echo(
echo   --- Глобальный конфиг %CFG_USER% ---
if exist "%CFG_USER%" type "%CFG_USER%"
if not exist "%CFG_USER%" echo   Файл отсутствует.
echo(
if defined CFG_HOME if exist "%CFG_HOME%" echo   --- Конфиг %CFG_HOME% ---
if defined CFG_HOME if exist "%CFG_HOME%" type "%CFG_HOME%"
echo(
echo   --- Переменные окружения CARGO_ ---
set CARGO_ 2>nul
echo(
echo   --- Доступ к crates.io ---
powershell -NoProfile -Command "try { $r = Invoke-WebRequest -UseBasicParsing -Uri 'https://index.crates.io/config.json' -TimeoutSec 20; Write-Host ('   OK, HTTP ' + $r.StatusCode) } catch { Write-Host ('   FAIL: ' + $_.Exception.Message) }"
pause
goto menu

:product_desktop
echo(
echo   ========== ПРОДУКТ: WAWITY DESKTOP ==========
echo(
echo   [1/4] Фронтенд vite
if not exist node_modules call npm install || exit /b 1
call npm run build || exit /b 1
echo(
echo   [2/4] Rust GUI tauri
if exist target\release\wawity-app.exe del /q target\release\wawity-app.exe
cargo build -p wawity --release || exit /b 1
if not exist target\release\wawity-app.exe echo [X] wawity-app.exe not built & exit /b 1
echo(
echo   [3/4] Payload
call :stage_payload desktop || exit /b 1
echo(
echo   [4/4] Установщик
call :build_installer desktop Desktop || exit /b 1
exit /b 0

:product_cli
echo(
echo   ========== ПРОДУКТ: WAWITY CLI ==========
echo(
echo   [1/3] Rust CLI wawity-cli
if exist target\release\wawity.exe del /q target\release\wawity.exe
cargo build -p wawity-cli --release || exit /b 1
if not exist target\release\wawity.exe echo [X] wawity.exe not built & exit /b 1
echo(
echo   [2/3] Payload
call :stage_payload cli || exit /b 1
echo(
echo   [3/3] Установщик
call :build_installer cli CLI || exit /b 1
exit /b 0

:stage_payload
set VARIANT=%1
if not exist installer\payload mkdir installer\payload
if not exist installer\payload\MicrosoftEdgeWebView2Setup.exe (
  echo   [X] Нет installer\payload\MicrosoftEdgeWebView2Setup.exe
  echo       Скачайте WebView2 Evergreen Bootstrapper и положите его туда.
  exit /b 1
)
if not exist src-tauri\binaries\sing-box-x86_64.exe (
  echo   [X] Нет src-tauri\binaries\sing-box-x86_64.exe
  exit /b 1
)
if not exist src-tauri\binaries\wintun.dll (
  echo   [X] Нет src-tauri\binaries\wintun.dll
  exit /b 1
)
set STAGE=installer\payload\stage
if exist %STAGE% rd /s /q %STAGE%
mkdir %STAGE%\rulesets
copy /y src-tauri\binaries\sing-box-x86_64.exe %STAGE%\ >nul || exit /b 1
copy /y src-tauri\binaries\wintun.dll %STAGE%\ >nul || exit /b 1
copy /y src-tauri\rulesets\*.srs %STAGE%\rulesets\ >nul || exit /b 1
if "%VARIANT%"=="desktop" copy /y target\release\wawity-app.exe %STAGE%\WawityApp.exe >nul || exit /b 1
if "%VARIANT%"=="cli" copy /y target\release\wawity.exe %STAGE%\wawity.exe >nul || exit /b 1
if "%VARIANT%"=="cli" if not exist %STAGE%\wawity.exe echo [X] wawity.exe missing in payload & exit /b 1
if "%VARIANT%"=="desktop" if not exist %STAGE%\WawityApp.exe echo [X] WawityApp.exe missing in payload & exit /b 1
powershell -NoProfile -Command "Compress-Archive -Path 'installer\payload\stage\*' -DestinationPath 'installer\payload\app.zip' -Force" || exit /b 1
rd /s /q %STAGE%
exit /b 0

:build_installer
set VARIANT=%1
set SUFFIX=%2
set WAWITY_VARIANT=%VARIANT%
pushd installer
cargo build --release
set ERR=%errorlevel%
popd
set WAWITY_VARIANT=
if not "%ERR%"=="0" exit /b 1
if not exist %DIST% mkdir %DIST%
copy /y installer\target\release\WawitySetup.exe %DIST%\WawitySetup-%SUFFIX%.exe >nul || exit /b 1
if "%VARIANT%"=="cli" copy /y %DIST%\WawitySetup-CLI.exe %DIST%\WawitySetup-CLI-serverinstall.exe >nul
echo(
echo   Готово: %DIST%\WawitySetup-%SUFFIX%.exe
exit /b 0
