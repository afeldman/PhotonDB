#!/bin/bash
# Build macOS DMG for RethinkDB 3.0

set -e

VERSION="${GITHUB_REF_NAME#v}"
PACKAGE_NAME="RethinkDB"
APP_NAME="${PACKAGE_NAME}.app"
BUILD_DIR="build/macos"
DMG_NAME="${PACKAGE_NAME}-${VERSION}.dmg"

echo "Building macOS DMG v${VERSION}..."

# Create app bundle structure
mkdir -p "${BUILD_DIR}/${APP_NAME}/Contents/MacOS"
mkdir -p "${BUILD_DIR}/${APP_NAME}/Contents/Resources"

# Copy binary
cp bin/rethinkdb-macos-x86_64 "${BUILD_DIR}/${APP_NAME}/Contents/MacOS/rethinkdb"
chmod 755 "${BUILD_DIR}/${APP_NAME}/Contents/MacOS/rethinkdb"

# Create Info.plist
cat > "${BUILD_DIR}/${APP_NAME}/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>rethinkdb</string>
    <key>CFBundleIdentifier</key>
    <string>com.rethinkdb.rethinkdb</string>
    <key>CFBundleName</key>
    <string>${PACKAGE_NAME}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# Create launcher script (opens terminal and runs server)
cat > "${BUILD_DIR}/${APP_NAME}/Contents/MacOS/launcher" <<'EOF'
#!/bin/bash
BASEDIR=$(dirname "$0")
cd "$BASEDIR"

osascript -e 'tell application "Terminal"
    activate
    do script "'"$BASEDIR/rethinkdb"' serve --dev-mode; exit"
end tell'
EOF

chmod 755 "${BUILD_DIR}/${APP_NAME}/Contents/MacOS/launcher"

# Create PkgInfo
echo -n "APPL????" > "${BUILD_DIR}/${APP_NAME}/Contents/PkgInfo"

# Create README
cat > "${BUILD_DIR}/README.txt" <<'EOF'
RethinkDB 3.0 for macOS
=======================

Installation
------------

1. Drag RethinkDB.app to your Applications folder
2. Open Terminal and run:
   /Applications/RethinkDB.app/Contents/MacOS/rethinkdb --help

Quick Start
-----------

Start server:
  /Applications/RethinkDB.app/Contents/MacOS/rethinkdb serve --dev-mode

Or add to PATH:
  export PATH="/Applications/RethinkDB.app/Contents/MacOS:$PATH"
  rethinkdb serve --dev-mode

Access admin UI:
  http://localhost:8080/_admin

CLI Usage
---------

  rethinkdb db create myapp
  rethinkdb table create --db myapp users
  rethinkdb --help

Documentation
-------------

https://rethinkdb.com/docs
EOF

# Create uninstall script
cat > "${BUILD_DIR}/Uninstall.command" <<'EOF'
#!/bin/bash
echo "Uninstalling RethinkDB..."
sudo rm -rf /Applications/RethinkDB.app
sudo pkgutil --forget com.rethinkdb.rethinkdb || true
echo "RethinkDB uninstalled."
EOF

chmod 755 "${BUILD_DIR}/Uninstall.command"

# Create DMG
mkdir -p dist
hdiutil create -volname "${PACKAGE_NAME}" \
    -srcfolder "${BUILD_DIR}" \
    -ov -format UDZO \
    "dist/${DMG_NAME}"

echo "✅ DMG created: dist/${DMG_NAME}"
