#!/bin/bash
# Build Debian package for PhotonDB

set -e

VERSION="${GITHUB_REF_NAME#v}"
PACKAGE_NAME="rethinkdb"
ARCH="amd64"
BUILD_DIR="build/debian"

echo "Building Debian package v${VERSION}..."

# Create directory structure
mkdir -p "${BUILD_DIR}/DEBIAN"
mkdir -p "${BUILD_DIR}/usr/bin"
mkdir -p "${BUILD_DIR}/usr/share/doc/${PACKAGE_NAME}"
mkdir -p "${BUILD_DIR}/usr/share/man/man1"
mkdir -p "${BUILD_DIR}/etc/rethinkdb"
mkdir -p "${BUILD_DIR}/lib/systemd/system"
mkdir -p "${BUILD_DIR}/var/lib/rethinkdb"
mkdir -p "${BUILD_DIR}/var/log/rethinkdb"

# Copy binary
cp bin/photondb-linux-x86_64 "${BUILD_DIR}/usr/bin/photondb"
chmod 755 "${BUILD_DIR}/usr/bin/photondb"

# Create control file
cat > "${BUILD_DIR}/DEBIAN/control" <<EOF
Package: ${PACKAGE_NAME}
Version: ${VERSION}
Section: database
Priority: optional
Architecture: ${ARCH}
Maintainer: Anton Feldmann <anton.feldmann@gmail.com>
Description: PhotonDB - The Scientific Computing Database
 A modern, real-time database written in Rust.
 Features include real-time changefeeds, horizontal scaling,
 and a powerful query language.
Homepage: https://rethinkdb.com
EOF

# Post-install script
cat > "${BUILD_DIR}/DEBIAN/postinst" <<'EOF'
#!/bin/bash
set -e

# Create rethinkdb user if not exists
if ! getent passwd rethinkdb > /dev/null; then
    useradd --system --user-group --home /var/lib/rethinkdb --shell /bin/false rethinkdb
fi

# Set permissions
chown -R rethinkdb:rethinkdb /var/lib/rethinkdb
chown -R rethinkdb:rethinkdb /var/log/rethinkdb

# Enable and start service
systemctl daemon-reload
systemctl enable rethinkdb.service || true

echo "RethinkDB installed successfully!"
echo "Start with: sudo systemctl start rethinkdb"
EOF

chmod 755 "${BUILD_DIR}/DEBIAN/postinst"

# Pre-remove script
cat > "${BUILD_DIR}/DEBIAN/prerm" <<'EOF'
#!/bin/bash
set -e

# Stop service if running
systemctl stop rethinkdb.service || true
systemctl disable rethinkdb.service || true
EOF

chmod 755 "${BUILD_DIR}/DEBIAN/prerm"

# Systemd service
cat > "${BUILD_DIR}/lib/systemd/system/rethinkdb.service" <<'EOF'
[Unit]
Description=PhotonDB Server
After=network.target

[Service]
Type=simple
User=rethinkdb
Group=rethinkdb
Environment="PHOTONDB_DATA=/var/lib/rethinkdb"
Environment="PHOTONDB_LOG_DIR=/var/log/rethinkdb"
ExecStart=/usr/bin/photondb serve --bind 0.0.0.0 --port 28015
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# Documentation
cat > "${BUILD_DIR}/usr/share/doc/${PACKAGE_NAME}/README.Debian" <<'EOF'
PhotonDB for Debian
========================

Quick Start
-----------

1. Start the service:
   sudo systemctl start rethinkdb

2. Enable on boot:
   sudo systemctl enable rethinkdb

3. Check status:
   sudo systemctl status rethinkdb

4. Access the admin UI:
   http://localhost:8080/_admin

Configuration
-------------

Data directory: /var/lib/rethinkdb
Log directory: /var/log/rethinkdb
Service file: /lib/systemd/system/rethinkdb.service

CLI Usage
---------

List databases:
  photondb db list

Create database:
  photondb db create myapp

Create table:
  photondb table create --db myapp users

See 'rethinkdb --help' for more commands.

Documentation
-------------

https://rethinkdb.com/docs
EOF

# Copyright
cat > "${BUILD_DIR}/usr/share/doc/${PACKAGE_NAME}/copyright" <<EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: rethinkdb
Source: https://github.com/rethinkdb/rethinkdb

Files: *
Copyright: 2025 Anton Feldmann <anton.feldmann@gmail.com>
License: Apache-2.0

License: Apache-2.0
 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at
 .
     http://www.apache.org/licenses/LICENSE-2.0
 .
 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
EOF

# Build package
mkdir -p dist
dpkg-deb --build "${BUILD_DIR}" "dist/${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"

echo "✅ Debian package created: dist/${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"
