# Howler Desktop Packaging

This directory contains build scripts for packaging Howler for different desktop platforms.

## Linux

Run the Linux build script:

```bash
cd packaging/linux
./build.sh
```

This will create:
- `.deb` package (if `dpkg-deb` is available)
- `.tar.gz` archive

## macOS

Run the macOS build script:

```bash
cd packaging/macos
./build.sh
```

This will create:
- `.app` bundle
- `.tar.gz` archive
- `.dmg` image (if `hdiutil` is available)

## Windows

Run the Windows build script (PowerShell):

```powershell
cd packaging\windows
.\build.ps1
```

This will create:
- `.zip` archive
- `.exe` installer (if Inno Setup is available)

## Requirements

- Rust and Cargo
- Platform-specific tools:
  - Linux: `dpkg-deb` (for .deb packages)
  - macOS: `hdiutil` (for .dmg images)
  - Windows: Inno Setup (for .exe installers)

## Version

Set the version via environment variable:

```bash
VERSION=1.0.0 ./build.sh
```

Or on Windows:

```powershell
$env:VERSION="1.0.0"
.\build.ps1
```
