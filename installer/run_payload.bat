@echo off
setlocal
cd /d "%~dp0"
echo ============================================
echo  Wawity installer full build
echo ============================================
echo.
echo [1/3] Payload...
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0make_payload.ps1" -NoPause
if errorlevel 1 goto fail

echo.
echo [2/3] Icon...
if exist "wawity.ico" (
  echo ok
) else (
  if exist "..\src-tauri\icons\wawity.ico" (
    copy /y "..\src-tauri\icons\wawity.ico" "wawity.ico" >nul
    echo copied from src-tauri\icons
  ) else (
    echo ERROR: wawity.ico not found, put it next to this bat
    goto fail
  )
)

echo.
echo [3/3] cargo build --release...
cargo build --release
if errorlevel 1 goto fail

echo.
echo ============================================
echo  DONE: target\release\WawitySetup.exe
echo ============================================
echo.
pause
exit /b 0

:fail
echo.
echo ============================================
echo  BUILD FAILED, read the error above
echo ============================================
echo.
pause
exit /b 1
