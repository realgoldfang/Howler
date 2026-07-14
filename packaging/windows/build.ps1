# Build script for Windows packaging

$ErrorActionPreference = "Stop"

$VERSION = if ($env:VERSION) { $env:VERSION } else { "0.1.0" }
$APP_NAME = "Howler"
$BUILD_DIR = "target\release"
$PACKAGE_DIR = "pkg\windows"

Write-Host "Building Howler for Windows..."

# Build the release binary
cargo build --release

# Create package directory
New-Item -ItemType Directory -Force -Path $PACKAGE_DIR

# Copy binaries
Copy-Item "$BUILD_DIR\howler-cli.exe" $PACKAGE_DIR\
Copy-Item "$BUILD_DIR\howler-tui.exe" $PACKAGE_DIR\
Copy-Item "$BUILD_DIR\howler-gui.exe" $PACKAGE_DIR\

# Create .zip package
Write-Host "Creating .zip package..."
Compress-Archive -Path "$PACKAGE_DIR\*" -DestinationPath "howler_${VERSION}_windows.zip"
Write-Host "Created .zip package: howler_${VERSION}_windows.zip"

# Create installer using Inno Setup if available
if (Get-Command "iscc" -ErrorAction SilentlyContinue) {
    Write-Host "Creating installer..."
    @"
[Setup]
AppName=$APP_NAME
AppVersion=$VERSION
DefaultDirName={commonpf}\Howler
DefaultGroupName=Howler
OutputBaseFilename=howler_${VERSION}_windows_setup
Compression=lzma
SolidCompression=yes

[Files]
Source: "$PACKAGE_DIR\howler-cli.exe"; DestDir: "{app}"
Source: "$PACKAGE_DIR\howler-tui.exe"; DestDir: "{app}"
Source: "$PACKAGE_DIR\howler-gui.exe"; DestDir: "{app}"

[Icons]
Name: "{group}\Howler GUI"; Filename: "{app}\howler-gui.exe"
Name: "{group}\Howler CLI"; Filename: "{app}\howler-cli.exe"
Name: "{group}\Howler TUI"; Filename: "{app}\howler-tui.exe"
Name: "{commondesktop}\Howler"; Filename: "{app}\howler-gui.exe"
"@ | Out-File -FilePath "installer.iss" -Encoding ASCII
    
    iscc installer.iss
    Remove-Item installer.iss
    Write-Host "Created installer: howler_${VERSION}_windows_setup.exe"
}

Write-Host "Windows packaging complete!"
