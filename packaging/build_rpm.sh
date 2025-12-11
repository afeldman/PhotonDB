#!/bin/bash
# Build RPM package for RethinkDB 3.0

set -e

VERSION="${GITHUB_REF_NAME#v}"
RELEASE="1"
PACKAGE_NAME="rethinkdb"
ARCH="x86_64"
BUILD_DIR="build/rpm"

echo "Building RPM package v${VERSION}..."

# Install rpmbuild if needed
if ! command -v rpmbuild &> /dev/null; then
    sudo apt-get update
    sudo apt-get install -y rpm
fi

# Create RPM build structure
mkdir -p "${BUILD_DIR}"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}
mkdir -p "${BUILD_DIR}/BUILD/${PACKAGE_NAME}-${VERSION}"

# Copy binary
mkdir -p "${BUILD_DIR}/BUILD/${PACKAGE_NAME}-${VERSION}/usr/bin"
cp bin/rethinkdb-linux-x86_64 "${BUILD_DIR}/BUILD/${PACKAGE_NAME}-${VERSION}/usr/bin/rethinkdb"
chmod 755 "${BUILD_DIR}/BUILD/${PACKAGE_NAME}-${VERSION}/usr/bin/rethinkdb"

# Create spec file
cat > "${BUILD_DIR}/SPECS/${PACKAGE_NAME}.spec" <<EOF
Name:           ${PACKAGE_NAME}
Version:        ${VERSION}
Release:        ${RELEASE}%{?dist}
Summary:        RethinkDB 3.0 - The Scientific Computing Database
License:        Apache-2.0
URL:            https://rethinkdb.com
BuildArch:      ${ARCH}

%description
RethinkDB is a modern, real-time database written in Rust.
Features include real-time changefeeds, horizontal scaling,
and a powerful query language.

%prep
# No prep needed for pre-built binary

%build
# No build needed for pre-built binary

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/etc/rethinkdb
mkdir -p %{buildroot}/usr/lib/systemd/system
mkdir -p %{buildroot}/var/lib/rethinkdb
mkdir -p %{buildroot}/var/log/rethinkdb

install -m 755 ../BUILD/${PACKAGE_NAME}-${VERSION}/usr/bin/rethinkdb %{buildroot}/usr/bin/rethinkdb

# Systemd service
cat > %{buildroot}/usr/lib/systemd/system/rethinkdb.service <<'SVCEOF'
[Unit]
Description=RethinkDB 3.0 Server
After=network.target

[Service]
Type=simple
User=rethinkdb
Group=rethinkdb
Environment="RETHINKDB_DATA=/var/lib/rethinkdb"
Environment="RETHINKDB_LOG_DIR=/var/log/rethinkdb"
ExecStart=/usr/bin/rethinkdb serve --bind 0.0.0.0 --port 28015
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
SVCEOF

%files
/usr/bin/rethinkdb
/usr/lib/systemd/system/rethinkdb.service
%dir /var/lib/rethinkdb
%dir /var/log/rethinkdb

%pre
# Create rethinkdb user
getent group rethinkdb >/dev/null || groupadd -r rethinkdb
getent passwd rethinkdb >/dev/null || \
    useradd -r -g rethinkdb -d /var/lib/rethinkdb -s /sbin/nologin \
    -c "RethinkDB Server" rethinkdb
exit 0

%post
# Set permissions
chown -R rethinkdb:rethinkdb /var/lib/rethinkdb
chown -R rethinkdb:rethinkdb /var/log/rethinkdb

# Enable service
systemctl daemon-reload
systemctl enable rethinkdb.service || true

echo "RethinkDB installed successfully!"
echo "Start with: sudo systemctl start rethinkdb"

%preun
# Stop service
if [ \$1 -eq 0 ]; then
    systemctl stop rethinkdb.service || true
    systemctl disable rethinkdb.service || true
fi

%postun
# Reload systemd
if [ \$1 -eq 0 ]; then
    systemctl daemon-reload
fi

%changelog
* $(date "+%a %b %d %Y") Anton Feldmann <anton.feldmann@gmail.com> - ${VERSION}-${RELEASE}
- Release ${VERSION}
EOF

# Build RPM
cd "${BUILD_DIR}"
rpmbuild --define "_topdir $(pwd)" -bb SPECS/${PACKAGE_NAME}.spec

# Copy to dist
mkdir -p ../../dist
cp RPMS/${ARCH}/${PACKAGE_NAME}-${VERSION}-${RELEASE}.*.rpm ../../dist/

echo "✅ RPM package created: dist/${PACKAGE_NAME}-${VERSION}-${RELEASE}.${ARCH}.rpm"
