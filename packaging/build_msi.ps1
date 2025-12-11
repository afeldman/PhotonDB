# Build Windows MSI Installer for RethinkDB 3.0
# PowerShell script

$ErrorActionPreference = "Stop"

$VERSION = $env:GITHUB_REF_NAME -replace '^v', ''
$PACKAGE_NAME = "RethinkDB"
$BUILD_DIR = "build\windows"
$MSI_NAME = "RethinkDB-$VERSION-x64.msi"

Write-Host "Building Windows MSI v$VERSION..."

# Install WiX if needed
if (-not (Get-Command candle -ErrorAction SilentlyContinue)) {
    Write-Host "Installing WiX Toolset..."
    choco install wixtoolset -y
    $env:PATH += ";C:\Program Files (x86)\WiX Toolset v3.11\bin"
}

# Create directory structure
New-Item -ItemType Directory -Force -Path "$BUILD_DIR\bin"
New-Item -ItemType Directory -Force -Path "$BUILD_DIR\docs"
New-Item -ItemType Directory -Force -Path "dist"

# Copy binary
Copy-Item "bin\rethinkdb-windows-x86_64.exe" "$BUILD_DIR\bin\rethinkdb.exe"

# Create README
@"
RethinkDB 3.0 for Windows
=========================

Quick Start
-----------

1. Open PowerShell or Command Prompt
2. Navigate to installation directory:
   cd "C:\Program Files\RethinkDB"

3. Start server:
   .\rethinkdb.exe serve --dev-mode

4. Access admin UI:
   http://localhost:8080/_admin

CLI Usage
---------

Create database:
  rethinkdb.exe db create myapp

Create table:
  rethinkdb.exe table create --db myapp users

List databases:
  rethinkdb.exe db list

See help:
  rethinkdb.exe --help

Data Location
-------------

Default data directory: %APPDATA%\RethinkDB\data
Default log directory: %APPDATA%\RethinkDB\logs

You can customize with:
  rethinkdb.exe serve --data-dir C:\path\to\data

Service Installation
--------------------

To run as Windows service:

1. Install NSSM (Non-Sucking Service Manager):
   choco install nssm

2. Install service:
   nssm install RethinkDB "C:\Program Files\RethinkDB\rethinkdb.exe"
   nssm set RethinkDB AppParameters serve --bind 0.0.0.0 --port 28015
   nssm set RethinkDB AppDirectory "C:\Program Files\RethinkDB"

3. Start service:
   nssm start RethinkDB

Documentation
-------------

https://rethinkdb.com/docs
"@ | Out-File -FilePath "$BUILD_DIR\docs\README.txt" -Encoding UTF8

# Create WiX source
@"
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" 
           Name="$PACKAGE_NAME" 
           Language="1033" 
           Version="$VERSION.0" 
           Manufacturer="RethinkDB" 
           UpgradeCode="12345678-1234-1234-1234-123456789012">
    
    <Package InstallerVersion="200" 
             Compressed="yes" 
             InstallScope="perMachine" 
             Platform="x64" />

    <MajorUpgrade DowngradeErrorMessage="A newer version of [ProductName] is already installed." />
    
    <MediaTemplate EmbedCab="yes" />

    <Feature Id="ProductFeature" Title="RethinkDB" Level="1">
      <ComponentGroupRef Id="ProductComponents" />
      <ComponentRef Id="ApplicationShortcut" />
      <ComponentRef Id="EnvironmentPath" />
    </Feature>

    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFiles64Folder">
        <Directory Id="INSTALLFOLDER" Name="RethinkDB">
          <Directory Id="BinFolder" Name="bin" />
          <Directory Id="DocsFolder" Name="docs" />
        </Directory>
      </Directory>
      
      <Directory Id="ProgramMenuFolder">
        <Directory Id="ApplicationProgramsFolder" Name="RethinkDB"/>
      </Directory>
    </Directory>

    <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
      <Component Id="MainExecutable" Guid="*" Win64="yes">
        <File Id="RethinkDBExe" 
              Source="$BUILD_DIR\bin\rethinkdb.exe" 
              KeyPath="yes" 
              Checksum="yes" />
      </Component>
      
      <Component Id="Documentation" Guid="*" Win64="yes">
        <File Id="ReadmeTxt" 
              Source="$BUILD_DIR\docs\README.txt" 
              KeyPath="yes" />
      </Component>
    </ComponentGroup>

    <DirectoryRef Id="ApplicationProgramsFolder">
      <Component Id="ApplicationShortcut" Guid="*" Win64="yes">
        <Shortcut Id="ApplicationStartMenuShortcut"
                  Name="RethinkDB"
                  Description="RethinkDB Database Server"
                  Target="[INSTALLFOLDER]bin\rethinkdb.exe"
                  WorkingDirectory="INSTALLFOLDER"/>
        <RemoveFolder Id="CleanUpShortCut" Directory="ApplicationProgramsFolder" On="uninstall"/>
        <RegistryValue Root="HKCU" 
                      Key="Software\RethinkDB" 
                      Name="installed" 
                      Type="integer" 
                      Value="1" 
                      KeyPath="yes"/>
      </Component>
    </DirectoryRef>

    <Component Id="EnvironmentPath" Directory="BinFolder" Guid="*" Win64="yes">
      <Environment Id="PATH" 
                   Name="PATH" 
                   Value="[BinFolder]" 
                   Permanent="no" 
                   Part="last" 
                   Action="set" 
                   System="yes" />
    </Component>

  </Product>
</Wix>
"@ | Out-File -FilePath "$BUILD_DIR\RethinkDB.wxs" -Encoding UTF8

# Build MSI
Push-Location $BUILD_DIR
try {
    Write-Host "Compiling WiX source..."
    candle.exe RethinkDB.wxs -arch x64 -out RethinkDB.wixobj
    
    Write-Host "Linking MSI..."
    light.exe RethinkDB.wixobj -out "..\..\dist\$MSI_NAME" -ext WixUIExtension
    
    Write-Host "✅ MSI created: dist\$MSI_NAME"
} finally {
    Pop-Location
}
