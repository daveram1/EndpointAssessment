# Build Windows MSI installer for Endpoint Assessment Agent
# Requires: WiX Toolset v3 (https://wixtoolset.org/)
# Run from: packaging/windows/wix directory

param(
    [string]$Version = "0.1.0",
    [string]$Architecture = "x64"
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Resolve-Path "$ScriptDir\..\..\..\"
$AgentExe = "$ProjectRoot\target\release\agent.exe"
$OutputDir = "$ProjectRoot\target\release"

Write-Host "Building Endpoint Assessment Agent MSI Installer" -ForegroundColor Cyan
Write-Host "Version: $Version"
Write-Host "Architecture: $Architecture"

# Check prerequisites
if (-not (Get-Command candle.exe -ErrorAction SilentlyContinue)) {
    Write-Error "WiX Toolset not found. Please install from https://wixtoolset.org/"
    exit 1
}

if (-not (Test-Path $AgentExe)) {
    Write-Host "Agent executable not found. Building..." -ForegroundColor Yellow
    Push-Location $ProjectRoot
    cargo build --release -p agent
    Pop-Location
}

if (-not (Test-Path $AgentExe)) {
    Write-Error "Failed to build agent executable"
    exit 1
}

# Update version in WXS file
$WxsContent = Get-Content "$ScriptDir\main.wxs" -Raw
$WxsContent = $WxsContent -replace 'Version="[0-9.]+"', "Version=`"$Version`""
$WxsContent | Set-Content "$ScriptDir\main.wxs"

# Compile WiX source
Write-Host "Compiling WiX source..." -ForegroundColor Yellow
candle.exe -arch $Architecture `
    -dAgentPath="$AgentExe" `
    -out "$OutputDir\agent.wixobj" `
    "$ScriptDir\main.wxs"

if ($LASTEXITCODE -ne 0) {
    Write-Error "WiX compilation failed"
    exit 1
}

# Link to create MSI
Write-Host "Linking MSI..." -ForegroundColor Yellow
$MsiName = "endpoint-agent-$Version-windows-$Architecture.msi"
light.exe -ext WixUIExtension `
    -out "$OutputDir\$MsiName" `
    "$OutputDir\agent.wixobj"

if ($LASTEXITCODE -ne 0) {
    Write-Error "WiX linking failed"
    exit 1
}

# Cleanup
Remove-Item "$OutputDir\agent.wixobj" -ErrorAction SilentlyContinue
Remove-Item "$OutputDir\endpoint-agent-*.wixpdb" -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "MSI created successfully: target\release\$MsiName" -ForegroundColor Green
Write-Host ""
Write-Host "To install:" -ForegroundColor Cyan
Write-Host "  msiexec /i $MsiName SERVER_URL=http://your-server:8080 AGENT_SECRET=your-secret"
