@echo off
setlocal
rem Builds the Linux release programs (build-release.sh) in WSL's default Linux distribution,
rem into dist\linux: the game and the dedicated server, with release-secret.txt's secret.
rem docs\release\linux.md sets the distribution up. Arguments go to build-release.sh
rem (game or server builds only that one).
pushd "%~dp0"
if errorlevel 1 (
    echo Could not open the project directory.
    pause
    exit /b 1
)
echo Close any running skate2-pc window before compiling: the build takes the processor.
echo.
wsl.exe --cd "%~dp0." -- sh ./build-release.sh %*
set "BUILD_RESULT=%ERRORLEVEL%"
echo.
if "%BUILD_RESULT%"=="0" (
    echo BUILD SUCCEEDED: dist\linux
) else (
    echo BUILD FAILED with exit code %BUILD_RESULT%. See the errors above.
)
popd
pause
exit /b %BUILD_RESULT%
