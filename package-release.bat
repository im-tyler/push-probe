@echo off
setlocal
pushd "%~dp0"
if errorlevel 1 (
    echo Could not open the project directory.
    pause
    exit /b 1
)

rem A release's update files in dist\ (docs\release\auto-update.md): the Windows zip of
rem target\release\skate2-pc.exe (build-release.bat), the Linux zip of dist\linux\skate2-pc and
rem dist\linux\skate2-server (build-release-linux.bat, or build-release.sh on Linux), and
rem latest-release.txt with its signature, which release-secret.txt signs. Upload all four to the
rem GitHub release tagged v<version>.
if not exist "target\release\skate2-pc.exe" (
    echo Build the Windows game first: build-release.bat
    popd
    exit /b 1
)
set "LINUX_MISSING="
if not exist "dist\linux\skate2-pc" set "LINUX_MISSING=1"
if not exist "dist\linux\skate2-server" set "LINUX_MISSING=1"
if defined LINUX_MISSING (
    echo Build the Linux programs first: build-release-linux.bat, or build-release.sh on Linux
    popd
    exit /b 1
)
if not exist "release-secret.txt" (
    echo release-secret.txt is missing: a release is signed with it.
    popd
    exit /b 1
)

set "CARGO_EXE=cargo"
where cargo >nul 2>&1
if errorlevel 1 set "CARGO_EXE=%USERPROFILE%\.cargo\bin\cargo.exe"
set "RELEASE="%CARGO_EXE%" run --release --quiet -p skate2-update --bin skate2-release --"

if not exist dist mkdir dist
%RELEASE% pack dist\skate2-windows-x64.zip target\release\skate2-pc.exe || goto failed
%RELEASE% pack dist\skate2-linux-x64.zip dist\linux\skate2-pc dist\linux\skate2-server || goto failed
%RELEASE% sign --out dist --windows dist\skate2-windows-x64.zip --linux dist\skate2-linux-x64.zip || goto failed
popd
exit /b 0

:failed
echo PACKAGING FAILED. See the errors above.
popd
exit /b 1
