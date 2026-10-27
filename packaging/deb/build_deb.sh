#!/usr/bin/env bash
set -e

VERSION="${1:-0.1.0}"
PACKAGE_NAME="shaft"
ARCH="amd64"
OUTPUT_DIR="target/debian"
BUILD_ROOT="${OUTPUT_DIR}/${PACKAGE_NAME}_${VERSION}_${ARCH}"

echo "==> Building Debian package for ${PACKAGE_NAME} v${VERSION} (${ARCH})..."

mkdir -p "${BUILD_ROOT}/DEBIAN"
mkdir -p "${BUILD_ROOT}/usr/bin"
mkdir -p "${BUILD_ROOT}/usr/share/applications"
mkdir -p "${BUILD_ROOT}/usr/share/icons/hicolor/32x32/apps"
mkdir -p "${BUILD_ROOT}/usr/share/icons/hicolor/128x128/apps"
mkdir -p "${BUILD_ROOT}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${BUILD_ROOT}/usr/share/icons/hicolor/512x512/apps"
mkdir -p "${BUILD_ROOT}/usr/share/icons/hicolor/scalable/apps"

# Find or build release binary
if [ -f "target/release/shaft" ]; then
    BINARY_PATH="target/release/shaft"
elif [ -f "target/release/shaft-app" ]; then
    BINARY_PATH="target/release/shaft-app"
else
    echo "==> Release binary not found. Building shaft in release mode..."
    npm run build
    cargo build --release -p shaft-app
    cp target/release/shaft-app target/release/shaft
    strip target/release/shaft || true
    BINARY_PATH="target/release/shaft"
fi

install -m 755 "${BINARY_PATH}" "${BUILD_ROOT}/usr/bin/shaft"
install -m 644 "packaging/shaft.desktop" "${BUILD_ROOT}/usr/share/applications/shaft.desktop"
install -m 644 "src-tauri/icons/32x32.png" "${BUILD_ROOT}/usr/share/icons/hicolor/32x32/apps/shaft.png"
install -m 644 "src-tauri/icons/128x128.png" "${BUILD_ROOT}/usr/share/icons/hicolor/128x128/apps/shaft.png"
install -m 644 "src-tauri/icons/128x128@2x.png" "${BUILD_ROOT}/usr/share/icons/hicolor/256x256/apps/shaft.png"
install -m 644 "src-tauri/icons/icon.png" "${BUILD_ROOT}/usr/share/icons/hicolor/512x512/apps/shaft.png"
install -m 644 "packaging/shaft.svg" "${BUILD_ROOT}/usr/share/icons/hicolor/scalable/apps/shaft.svg"

# Control file
cat <<EOF > "${BUILD_ROOT}/DEBIAN/control"
Package: ${PACKAGE_NAME}
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Depends: libwebkit2gtk-4.1-0 | libwebkit2gtk-4.0-37, libgtk-3-0
Recommends: ffmpeg
Maintainer: Larvance <https://github.com/larvance>
Description: Universal zero-click drag-and-drop file converter, media tool, and action wheel
 High-performance offline format transcoder and media processor in pure safe Rust and Tauri.
 Supports 275 conversion pathways across video, audio, image, document, spreadsheet,
 archive, and subtitle formats with radial gesture wheel.
EOF

# Post-installation script (MIME database, icon cache, and desktop database)
cat <<'EOF' > "${BUILD_ROOT}/DEBIAN/postinst"
#!/bin/sh
set -e

if [ "$1" = "configure" ]; then
    if which update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q /usr/share/applications || true
    fi
    if which update-mime-database >/dev/null 2>&1; then
        update-mime-database /usr/share/mime || true
    fi
    if which gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
    fi
fi

exit 0
EOF
chmod 755 "${BUILD_ROOT}/DEBIAN/postinst"

# Post-removal script
cat <<'EOF' > "${BUILD_ROOT}/DEBIAN/postrm"
#!/bin/sh
set -e

if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    if which update-desktop-database >/dev/null 2>&1; then
        update-desktop-database -q /usr/share/applications || true
    fi
    if which update-mime-database >/dev/null 2>&1; then
        update-mime-database /usr/share/mime || true
    fi
    if which gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor || true
    fi
fi

exit 0
EOF
chmod 755 "${BUILD_ROOT}/DEBIAN/postrm"

dpkg-deb --build --root-owner-group "${BUILD_ROOT}" "${OUTPUT_DIR}/${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"
echo "==> Successfully created: ${OUTPUT_DIR}/${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"
