#!/usr/bin/env bash
#
# Assemble the Slint desktop binary into a macOS .app bundle.
#
#   ./scripts/bundle-macos.sh [--debug]
#
# macOS reads an app's Dock and Finder icon from the bundle's Info.plist and
# .icns file -- it ignores the window icon that Slint's `Window.icon` sets, so
# a bare `cargo run` always shows a generic executable icon no matter what the
# .slint file says. Bundling is the only way to get the real icon on this
# platform, and it's also the prerequisite for signing and notarizing a
# distributable build later.
#
# The bundle produced here is unsigned. Gatekeeper will refuse to open it by
# double-click on another machine; right-click -> Open works locally.

set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

PROFILE=release
PROFILE_DIR=release
if [[ "${1:-}" == "--debug" ]]; then
  PROFILE=dev
  PROFILE_DIR=debug
fi

BIN_NAME=mortgage-ui-slint
APP_NAME="Mortgage Calculator"
BUNDLE_ID=io.github.dengf.mortgagecalculator
ICNS=assets/icon/icon-macos.icns

# Keep the bundle version in step with the workspace rather than hardcoding
# it, so a version bump can't silently apply to only some platforms.
VERSION=$(sed -n '/^\[workspace.package\]/,/^\[/p' Cargo.toml \
  | sed -n 's/^version = "\(.*\)"/\1/p' | head -1)
if [[ -z "$VERSION" ]]; then
  echo "error: could not read version from [workspace.package] in Cargo.toml" >&2
  exit 1
fi

if [[ ! -f "$ICNS" ]]; then
  echo "error: $ICNS is missing -- run 'python3 assets/icon/generate.py' first" >&2
  exit 1
fi

echo "Building $BIN_NAME ($PROFILE)..."
cargo build --profile "$PROFILE" -p mortgage-ui-slint --bin "$BIN_NAME"

APP="target/macos/$APP_NAME.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

cp "target/$PROFILE_DIR/$BIN_NAME" "$APP/Contents/MacOS/$BIN_NAME"
cp "$ICNS" "$APP/Contents/Resources/icon.icns"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>       <string>en</string>
  <key>CFBundleDisplayName</key>             <string>$APP_NAME</string>
  <key>CFBundleName</key>                    <string>$APP_NAME</string>
  <key>CFBundleIdentifier</key>              <string>$BUNDLE_ID</string>
  <key>CFBundleExecutable</key>              <string>$BIN_NAME</string>
  <key>CFBundleIconFile</key>                <string>icon</string>
  <key>CFBundleInfoDictionaryVersion</key>   <string>6.0</string>
  <key>CFBundlePackageType</key>             <string>APPL</string>
  <key>CFBundleShortVersionString</key>      <string>$VERSION</string>
  <key>CFBundleVersion</key>                 <string>$VERSION</string>
  <key>LSMinimumSystemVersion</key>          <string>11.0</string>
  <key>LSApplicationCategoryType</key>       <string>public.app-category.finance</string>
  <key>NSHighResolutionCapable</key>         <true/>
</dict>
</plist>
PLIST

# Finder caches bundle icons by mtime; without this the Dock can keep showing
# the previous icon after a rebuild.
touch "$APP"

echo
echo "Built $APP  (version $VERSION, unsigned)"
echo "Run it with:  open '$APP'"
