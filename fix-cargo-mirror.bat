@echo off
setlocal
chcp 65001 >nul
title Wawity: отключение зеркала cargo

set "CFG=%USERPROFILE%\.cargo\config.toml"
if not exist "%CFG%" set "CFG=%USERPROFILE%\.cargo\config"
if not exist "%CFG%" (
  echo   Глобальный конфиг cargo не найден, ничего делать не нужно.
  pause
  exit /b 0
)

echo   Текущий конфиг: %CFG%
echo(
type "%CFG%"
echo(
echo   Будет создана резервная копия config.bak, а сам конфиг переименован.
echo   После этого cargo будет ходить напрямую в crates.io.
echo(
set ANS=
set /p ANS=   Продолжить? [y/n]: 
if /i not "%ANS%"=="y" exit /b 0

copy /y "%CFG%" "%CFG%.bak" >nul
del /q "%CFG%"
echo(
echo   Готово. Резервная копия: %CFG%.bak
echo   Вернуть обратно: переименуйте .bak обратно в config.toml
pause
