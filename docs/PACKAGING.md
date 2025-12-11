# Packaging & Release Guide

This document describes how to build packages and create releases for PhotonDB.

## Overview

RethinkDB supports the following distribution formats:

- **Binaries:** Static executables for Linux, macOS, Windows
- **Debian/Ubuntu:** `.deb` packages
- **RPM/Fedora:** `.rpm` packages
- **macOS:** `.dmg` disk images
- **Windows:** `.msi` installers
- **Docker:** Multi-arch container images

## Automated Releases (Recommended)

### GitHub Actions

Releases are automatically built when you push a version tag:

```bash
# Create and push tag
git tag v3.0.0
git push origin v3.0.0
```

This triggers:

1. **CI Pipeline** - Tests, linting, security checks
2. **Cross-platform builds** - Linux, macOS, Windows (x86_64 + ARM64)
3. **Package creation** - .deb, .rpm, .dmg, .msi
4. **Docker images** - Multi-arch (amd64, arm64)
5. **GitHub Release** - Automatic release creation with assets

### What Gets Built

| Platform            | Artifact   | Name                           |
| ------------------- | ---------- | ------------------------------ |
| Linux x86_64        | Binary     | `rethinkdb-linux-x86_64`       |
| Linux ARM64         | Binary     | `rethinkdb-linux-aarch64`      |
| macOS Intel         | Binary     | `rethinkdb-macos-x86_64`       |
| macOS Apple Silicon | Binary     | `rethinkdb-macos-aarch64`      |
| Windows x64         | Binary     | `rethinkdb-windows-x86_64.exe` |
| Debian/Ubuntu       | Package    | `photondb_3.0.0_amd64.deb`    |
| RPM/Fedora          | Package    | `rethinkdb-3.0.0-1.x86_64.rpm` |
| macOS               | Disk Image | `RethinkDB-3.0.0.dmg`          |
| Windows             | Installer  | `RethinkDB-3.0.0-x64.msi`      |
| Docker              | Image      | `rethinkdb/rethinkdb:3.0.0`    |

## Manual Builds

### Prerequisites

**Linux (Debian/Ubuntu):**

```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    curl \
    git \
    rpm \
    dpkg-dev
```

**macOS:**

```bash
brew install rust
```

**Windows:**

```powershell
# Install Rust via rustup
# Install WiX Toolset via Chocolatey
choco install wixtoolset
```

### Building Binaries

#### Release Build

```bash
# All platforms
cargo build --release --bin rethinkdb

# Binary location
./target/release/photondb  # Linux/macOS
./target/release/photondb.exe  # Windows
```

#### Cross-compilation

**Linux ARM64 from x86_64:**

```bash
# Install cross-compiler
sudo apt-get install gcc-aarch64-linux-gnu

# Add target
rustup target add aarch64-unknown-linux-gnu

# Build
cargo build --release --target aarch64-unknown-linux-gnu --bin rethinkdb
```

**macOS Universal Binary:**

```bash
# Add targets
rustup target add x86_64-apple-darwin aarch64-apple-darwin

# Build both
cargo build --release --target x86_64-apple-darwin --bin rethinkdb
cargo build --release --target aarch64-apple-darwin --bin rethinkdb

# Create universal binary
lipo -create \
    target/x86_64-apple-darwin/release/photondb \
    target/aarch64-apple-darwin/release/photondb \
    -output rethinkdb-universal
```

### Building Packages

#### Debian Package

```bash
# Set version
export GITHUB_REF_NAME=v3.0.0

# Build binary first
cargo build --release --bin rethinkdb
mkdir -p bin
cp target/release/photondb bin/photondb-linux-x86_64

# Build package
./packaging/build_deb.sh

# Output: dist/photondb_3.0.0_amd64.deb
```

**Test installation:**

```bash
sudo dpkg -i dist/photondb_3.0.0_amd64.deb
rethinkdb --version
```

#### RPM Package

```bash
# Set version
export GITHUB_REF_NAME=v3.0.0

# Build binary first
cargo build --release --bin rethinkdb
mkdir -p bin
cp target/release/photondb bin/photondb-linux-x86_64

# Build package
./packaging/build_rpm.sh

# Output: dist/rethinkdb-3.0.0-1.x86_64.rpm
```

**Test installation:**

```bash
sudo rpm -i dist/rethinkdb-3.0.0-1.x86_64.rpm
rethinkdb --version
```

#### macOS DMG

```bash
# Set version
export GITHUB_REF_NAME=v3.0.0

# Build binary first (Universal)
cargo build --release --target x86_64-apple-darwin --bin rethinkdb
mkdir -p bin
cp target/x86_64-apple-darwin/release/photondb bin/photondb-macos-x86_64

# Build DMG
./packaging/build_dmg.sh

# Output: dist/RethinkDB-3.0.0.dmg
```

**Test installation:**

```bash
open dist/RethinkDB-3.0.0.dmg
# Drag to Applications
/Applications/RethinkDB.app/Contents/MacOS/rethinkdb --version
```

#### Windows MSI

```powershell
# Set version
$env:GITHUB_REF_NAME = "v3.0.0"

# Build binary first
cargo build --release --bin rethinkdb
New-Item -ItemType Directory -Force -Path bin
Copy-Item target\release\rethinkdb.exe bin\rethinkdb-windows-x86_64.exe

# Build MSI
.\packaging\build_msi.ps1

# Output: dist\RethinkDB-3.0.0-x64.msi
```

**Test installation:**

```powershell
msiexec /i dist\RethinkDB-3.0.0-x64.msi
rethinkdb --version
```

### Docker Images

#### Build

```bash
# Single architecture
docker build -t rethinkdb/rethinkdb:3.0.0 .

# Multi-architecture (requires buildx)
docker buildx create --use
docker buildx build \
    --platform linux/amd64,linux/arm64 \
    -t rethinkdb/rethinkdb:3.0.0 \
    --push .
```

#### Test

```bash
docker run -p 8080:8080 rethinkdb/rethinkdb:3.0.0 serve --dev-mode
```

## Release Checklist

### Pre-Release

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md` with release notes
- [ ] Update documentation
- [ ] Run full test suite: `cargo test --all-features`
- [ ] Run benchmarks: `cargo bench`
- [ ] Update Docker image tags

### Creating Release

1. **Create release branch:**

   ```bash
   git checkout -b release/3.0.0
   ```

2. **Update versions:**

   ```bash
   # Cargo.toml
   version = "3.0.0"
   ```

3. **Commit changes:**

   ```bash
   git add .
   git commit -m "Release v3.0.0"
   git push origin release/3.0.0
   ```

4. **Create and push tag:**

   ```bash
   git tag -a v3.0.0 -m "Release v3.0.0"
   git push origin v3.0.0
   ```

5. **Wait for CI/CD:**

   - GitHub Actions will automatically build all packages
   - Check: https://github.com/rethinkdb/rethinkdb/actions

6. **Verify release:**
   - GitHub Releases page should have new release
   - All artifacts should be attached
   - Docker images should be pushed

### Post-Release

- [ ] Announce on website/blog
- [ ] Update documentation site
- [ ] Create Docker Hub release notes
- [ ] Update package repositories (Homebrew, Chocolatey, etc.)
- [ ] Social media announcements

## Package Details

### Debian Package

**Install locations:**

- Binary: `/usr/bin/photondb`
- Data: `/var/lib/rethinkdb`
- Logs: `/var/log/rethinkdb`
- Service: `/lib/systemd/system/rethinkdb.service`

**User/Group:**

- User: `rethinkdb` (created automatically)
- Group: `rethinkdb`

**Start service:**

```bash
sudo systemctl start rethinkdb
sudo systemctl enable rethinkdb
```

### RPM Package

**Install locations:**

- Binary: `/usr/bin/photondb`
- Data: `/var/lib/rethinkdb`
- Logs: `/var/log/rethinkdb`
- Service: `/usr/lib/systemd/system/rethinkdb.service`

**Start service:**

```bash
sudo systemctl start rethinkdb
sudo systemctl enable rethinkdb
```

### macOS DMG

**Install locations:**

- App Bundle: `/Applications/RethinkDB.app`
- Binary: `/Applications/RethinkDB.app/Contents/MacOS/rethinkdb`

**Add to PATH:**

```bash
export PATH="/Applications/RethinkDB.app/Contents/MacOS:$PATH"
```

### Windows MSI

**Install locations:**

- Program: `C:\Program Files\RethinkDB\`
- Binary: `C:\Program Files\RethinkDB\bin\rethinkdb.exe`
- Data: `%APPDATA%\RethinkDB\data`
- Logs: `%APPDATA%\RethinkDB\logs`

**Added to PATH automatically**

## Troubleshooting

### Build Fails

**Error: `linker 'cc' not found`**

```bash
# Install build tools
sudo apt-get install build-essential  # Linux
xcode-select --install  # macOS
```

**Error: `failed to run custom build command for openssl-sys`**

```bash
# Install OpenSSL dev
sudo apt-get install libssl-dev pkg-config  # Linux
brew install openssl  # macOS
```

### Package Build Fails

**Debian: `dpkg-deb: error`**

- Check file permissions in build directory
- Ensure all directories have correct ownership

**RPM: `rpmbuild not found`**

```bash
sudo apt-get install rpm  # Ubuntu/Debian
sudo yum install rpm-build  # CentOS/RHEL
```

**macOS: `hdiutil: create failed`**

- Check disk space
- Ensure no existing mount points

**Windows: `candle.exe not found`**

```powershell
# Add WiX to PATH
$env:PATH += ";C:\Program Files (x86)\WiX Toolset v3.11\bin"
```

## CI/CD Configuration

### Required Secrets

GitHub repository secrets:

- `DOCKER_USERNAME` - Docker Hub username
- `DOCKER_PASSWORD` - Docker Hub password/token
- `CODECOV_TOKEN` - Codecov.io token (optional)

### Workflow Files

- `.github/workflows/ci.yml` - CI pipeline (lint, test, build)
- `.github/workflows/release.yml` - Release pipeline (triggered by tags)

## Version Numbering

RethinkDB uses Semantic Versioning (SemVer):

```
MAJOR.MINOR.PATCH[-PRERELEASE][+BUILDMETADATA]

Examples:
- 3.0.0        - Stable release
- 3.0.1        - Patch release
- 3.1.0        - Minor version
- 3.0.0-alpha  - Pre-release
- 3.0.0-beta.1 - Numbered pre-release
- 3.0.0-rc.1   - Release candidate
```

### Version Bumping

```bash
# Patch (bug fixes)
3.0.0 → 3.0.1

# Minor (new features, backward compatible)
3.0.0 → 3.1.0

# Major (breaking changes)
3.0.0 → 4.0.0
```

## Distribution Channels

### Official

- **GitHub Releases**: https://github.com/rethinkdb/rethinkdb/releases
- **Docker Hub**: https://hub.docker.com/r/rethinkdb/rethinkdb
- **GitHub Container Registry**: ghcr.io/rethinkdb/rethinkdb

### Community (Future)

- **Homebrew** (macOS): `brew install rethinkdb`
- **Chocolatey** (Windows): `choco install rethinkdb`
- **Snapcraft** (Linux): `snap install rethinkdb`
- **apt/yum repositories**: PPA for Ubuntu, Copr for Fedora

---

**Last Updated:** December 9, 2025
**Author:** Anton Feldmann
