#!/usr/bin/env bash
set -euo pipefail

# shaft unified release script
# Builds and uploads strictly:
# 1. shaft_<version>_amd64.AppImage
# 2. shaft-<version>-linux-x86_64.tar.gz
# 3. shaft_<version>_amd64.deb

REPO="larvance/shaft"
SOURCE="${BASH_SOURCE[0]}"
while [ -h "$SOURCE" ]; do
  DIR="$(cd -P "$(dirname "$SOURCE")" && pwd)"
  SOURCE="$(readlink "$SOURCE")"
  [[ $SOURCE != /* ]] && SOURCE="$DIR/$SOURCE"
done
SCRIPT_DIR="$(cd -P "$(dirname "$SOURCE")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"

# Determine target release tag / version
TAG="${1:-}"
if [ -z "${TAG}" ]; then
  TAG=$(gh release view -R "${REPO}" --json tagName -q .tagName 2>/dev/null || true)
  if [ -z "${TAG}" ]; then
    TAG=$(grep -m1 '^version' Cargo.toml | awk -F '"' '{print $2}')
  fi
fi

VERSION="${TAG#v}"
echo "==> Preparing release for shaft version ${VERSION} (tag: ${TAG})..."

# Ensure tools are present
command -v appimagetool >/dev/null 2>&1 || {
  echo "Error: appimagetool not found in PATH. Ensure ~/.local/bin is in PATH." >&2
  exit 1
}
command -v dpkg-deb >/dev/null 2>&1 || {
  echo "Error: dpkg-deb not found in PATH." >&2
  exit 1
}
command -v gh >/dev/null 2>&1 || {
  echo "Error: gh CLI not found in PATH." >&2
  exit 1
}

# Dedicated staging directory for release artifacts
DIST_DIR="${ROOT_DIR}/dist-release"
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}"

echo "==> Building frontend assets with Vite..."
npm run build

echo "==> Compiling shaft release binary..."
cargo build --release -p shaft-app
cp target/release/shaft-app target/release/shaft
strip target/release/shaft target/release/shaft-app || true

echo "==> Building Debian package..."
bash packaging/deb/build_deb.sh "${VERSION}"
cp "target/debian/shaft_${VERSION}_amd64.deb" "${DIST_DIR}/shaft_${VERSION}_amd64.deb"

echo "==> Assembling AppImage..."
APPDIR="${ROOT_DIR}/target/shaft_appdir"
rm -rf "${APPDIR}"
mkdir -p "${APPDIR}/usr/bin" \
         "${APPDIR}/usr/share/applications" \
         "${APPDIR}/usr/share/icons/hicolor/scalable/apps" \
         "${APPDIR}/usr/share/icons/hicolor/256x256/apps" \
         "${APPDIR}/usr/share/icons/hicolor/128x128/apps" \
         "${APPDIR}/usr/share/icons/hicolor/32x32/apps"

cp target/release/shaft "${APPDIR}/usr/bin/shaft"
chmod +x "${APPDIR}/usr/bin/shaft"
cp packaging/shaft.desktop "${APPDIR}/shaft.desktop"
cp packaging/shaft.desktop "${APPDIR}/usr/share/applications/shaft.desktop"
cp src-tauri/icons/128x128@2x.png "${APPDIR}/shaft.png"
cp src-tauri/icons/128x128@2x.png "${APPDIR}/usr/share/icons/hicolor/256x256/apps/shaft.png"
cp src-tauri/icons/128x128.png "${APPDIR}/usr/share/icons/hicolor/128x128/apps/shaft.png"
cp src-tauri/icons/32x32.png "${APPDIR}/usr/share/icons/hicolor/32x32/apps/shaft.png"
cp packaging/shaft.svg "${APPDIR}/usr/share/icons/hicolor/scalable/apps/shaft.svg"

cat <<'EOF' > "${APPDIR}/AppRun"
#!/bin/sh
HERE="$(dirname "$(readlink -f "${0}")")"
exec "${HERE}/usr/bin/shaft" "$@"
EOF
chmod +x "${APPDIR}/AppRun"

ARCH=x86_64 appimagetool "${APPDIR}" "${DIST_DIR}/shaft_${VERSION}_amd64.AppImage"

echo "==> Creating portable tarball..."
TAR_DIR="${ROOT_DIR}/target/tarball_staging"
rm -rf "${TAR_DIR}"
mkdir -p "${TAR_DIR}/icons"

cp target/release/shaft "${TAR_DIR}/shaft"
chmod +x "${TAR_DIR}/shaft"
cp install.sh "${TAR_DIR}/install.sh"
chmod +x "${TAR_DIR}/install.sh"
cp packaging/shaft.desktop "${TAR_DIR}/shaft.desktop"
cp README.md LICENSE "${TAR_DIR}/" 2>/dev/null || true
cp src-tauri/icons/128x128.png "${TAR_DIR}/icons/"
cp src-tauri/icons/32x32.png "${TAR_DIR}/icons/"
cp packaging/shaft.svg "${TAR_DIR}/icons/" 2>/dev/null || true

tar -czf "${DIST_DIR}/shaft-${VERSION}-linux-x86_64.tar.gz" -C "${TAR_DIR}" .

echo "==> Staged release assets in dist-release/:"
ls -lh "${DIST_DIR}"

# Check upload flag (default: true)
UPLOAD="${UPLOAD:-true}"
if [ "${UPLOAD}" = "true" ]; then
  echo "==> Uploading assets to GitHub release ${TAG} in ${REPO}..."
  if gh release view "${TAG}" -R "${REPO}" >/dev/null 2>&1; then
    gh release upload "${TAG}" "${DIST_DIR}"/* --clobber -R "${REPO}"
  else
    echo "Creating new release ${TAG}..."
    gh release create "${TAG}" "${DIST_DIR}"/* \
      --title "shaft ${VERSION}: Universal Zero-Click Drag-and-Drop File Converter & Action Wheel" \
      --notes "Universal zero-click, offline, hardware-accelerated drag-and-drop file converter, media tool, and action wheel in Rust." \
      -R "${REPO}"
  fi
  echo "==> Release assets successfully uploaded to ${REPO} @ ${TAG}!"
else
  echo "==> UPLOAD=false specified; skipping upload."
fi
