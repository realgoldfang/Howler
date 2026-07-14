#!/bin/bash
# Build script for Linux packaging

set -e

VERSION=${VERSION:-"0.1.0"}
APP_NAME="howler"
BUILD_DIR="target/release"
PACKAGE_DIR="pkg/linux"

echo "Building Howler for Linux..."

# Build the release binary
cargo build --release

# Create package directory
mkdir -p "$PACKAGE_DIR"

# Copy binary
cp "$BUILD_DIR/howler-cli" "$PACKAGE_DIR/"
cp "$BUILD_DIR/howler-tui" "$PACKAGE_DIR/"
cp "$BUILD_DIR/howler-gui" "$PACKAGE_DIR/"

# Create desktop entry
mkdir -p "$PACKAGE_DIR/usr/share/applications"
cat > "$PACKAGE_DIR/usr/share/applications/howler.desktop" << EOF
[Desktop Entry]
Name=Howler
Comment=Wolf tracking application
Exec=/usr/bin/howler-gui
Icon=howler
Terminal=false
Type=Application
Categories=Science;Education;
EOF

# Create .deb package (requires dpkg-deb)
if command -v dpkg-deb &> /dev/null; then
    echo "Creating .deb package..."
    mkdir -p "$PACKAGE_DIR/DEBIAN"
    cat > "$PACKAGE_DIR/DEBIAN/control" << EOF
Package: howler
Version: $VERSION
Section: science
Priority: optional
Maintainer: Howler Team
Architecture: amd64
Description: Wolf tracking application
 Howler is a desktop application for tracking and analyzing wolf
 sightings and movements across multiple data sources.
EOF
    
    dpkg-deb --build "$PACKAGE_DIR" "howler_${VERSION}_amd64.deb"
    echo "Created .deb package: howler_${VERSION}_amd64.deb"
fi

# Create .tar.gz package
echo "Creating .tar.gz package..."
tar -czf "howler_${VERSION}_linux.tar.gz" -C "$PACKAGE_DIR" .
echo "Created .tar.gz package: howler_${VERSION}_linux.tar.gz"

echo "Linux packaging complete!"
