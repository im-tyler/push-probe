@echo off
setlocal
pushd "%~dp0"
if errorlevel 1 (
    echo Could not open the project directory.
    pause
    exit /b 1
)

set "CARGO_EXE=cargo"
where cargo >nul 2>&1
if errorlevel 1 set "CARGO_EXE=%USERPROFILE%\.cargo\bin\cargo.exe"

rem release-secret.txt (tools\new_release_secret.py; keep it private) gives every build of one
rem version the same secret, so this game and the Linux server of build-release.sh meet.
if exist "release-secret.txt" set /p SKATE2_RELEASE_SECRET=<"release-secret.txt"
if defined SKATE2_RELEASE_SECRET (
    echo Release build: its secret comes from the release key and the version in Cargo.toml.
) else (
    echo Development build: no release-secret.txt, so its secret is random.
)

rem release-repository.txt names the GitHub repository (owner/name) whose releases a release
rem build updates to (docs\release\auto-update.md); a build without it never looks for updates.
if exist "release-repository.txt" set /p SKATE2_UPDATE_REPOSITORY=<"release-repository.txt"
if defined SKATE2_UPDATE_REPOSITORY (
    if defined SKATE2_RELEASE_SECRET echo Updates come from the releases of %SKATE2_UPDATE_REPOSITORY%.
) else (
    echo No release-repository.txt: this build does not look for updates.
)

rem Release binaries embed their dependencies' source paths. These name the profile folder, and
rem with it the Windows user name, so the build writes ~ in its place. The rustflags join the
rem ones in .cargo\config.toml, which RUSTFLAGS would replace.
set "REMAP='--remap-path-prefix=%USERPROFILE%=~'"
if defined CARGO_HOME set "REMAP=%REMAP%, '--remap-path-prefix=%CARGO_HOME%=cargo'"

echo Building Skate 2 in incremental release mode with one compiler job...
echo The first build fills the cache; later builds reuse it. Keep the target folder.
echo Close any running skate2-pc window before compiling.
echo.
"%CARGO_EXE%" build --release -p skate2-pc -j 1 --timings --config "target.x86_64-pc-windows-msvc.rustflags=[%REMAP%]"
set "BUILD_RESULT=%ERRORLEVEL%"
echo.
if "%BUILD_RESULT%"=="0" (
    echo BUILD SUCCEEDED: target\release\skate2-pc.exe
    echo Build timing report: target\cargo-timings\cargo-timing.html
) else (
    echo BUILD FAILED with exit code %BUILD_RESULT%. See the errors above.
)
popd
pause
exit /b %BUILD_RESULT%
