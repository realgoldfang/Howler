#!/bin/bash
# Build script for macOS packaging

set -e

VERSION=${VERSION:-"0.1.0"}
APP_NAME="Howler"
BUILD_DIR="target/release"
PACKAGE_DIR="pkg/macos"

echo "Building Howler for macOS..."

# Build the release binary
cargo build --release

# Create package directory
mkdir -p "$PACKAGE_DIR"

# Copy binaries
cp "$BUILD_DIR/howler-cli" "$PACKAGE_DIR/"
cp "$BUILD_DIR/howler-tui" "$PACKAGE_DIR/"
cp "$BUILD_DIR/hower-gui" "$PACKAGE_DIR/"

# Create .app bundle for GUI
APP_BUNDLE="$PACKAGE_DIR/${APP_NAME}.app"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

cat > "$APP_BUNDLE/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>howler-gui</string>
    <key>CFBundleIdentifier</key>
    <string>com.howler.app</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

cp "$BUILD_DIR/howler-gui" "$APP_BUNDLE/Contents/MacOS/"

# Create .tar.gz package
echo "Creating .tar.gz package..."
tar -czf "howler_${VERSION}_macos.tar.gz" -C "$PACKAGE_DIR" .
echo "Created .tar.gz package: howler_${VERSION}_macos.tar.gz"

# Create .dmg if hdiutil is available
if command -v hdiutil &> /dev/null; then
    echo "Creating .dmg image..."
    hdiutil create -volname "${APP_NAME}" -srcfolder "$APP_BUNDLE" -ov -format UDZO "howler_${VERSION}_macos.dmg"
    echo "Created .dmg image: howler_${VERSION}_macos.dmg"
fi

echo "macOS packaging complete!"
