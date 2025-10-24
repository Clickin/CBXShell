# NSIS Build Script for CBXShell
# This script builds separate x86 and x64 NSIS installers for GitHub releases

param(
    [string]$Configuration = "Release",
    [string]$NsisPath = "${env:ProgramFiles(x86)}\NSIS\makensis.exe",
    [ValidateSet("Both", "x86", "x64")]
    [string]$Architecture = "Both"
)

$ErrorActionPreference = "Stop"

Write-Host "Building CBXShell-rs NSIS Installer(s)" -ForegroundColor Cyan
Write-Host "Configuration: $Configuration" -ForegroundColor Gray
Write-Host "Architecture: $Architecture" -ForegroundColor Gray
Write-Host ""

# Check if NSIS is installed
if (-not (Test-Path $NsisPath)) {
    Write-Host "ERROR: NSIS not found at $NsisPath" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please install NSIS from:" -ForegroundColor Yellow
    Write-Host "  https://nsis.sourceforge.io/Download" -ForegroundColor Cyan
    Write-Host ""
    exit 1
}

# Check for UnRAR DLLs
Write-Host "[Prerequisites] Checking UnRAR DLLs..." -ForegroundColor Yellow
$UnrarDllX86 = "target\release\UnRAR.dll"
$UnrarDllX64 = "target\release\UnRAR64.dll"

$missingDlls = @()
if (($Architecture -eq "Both" -or $Architecture -eq "x86") -and -not (Test-Path $UnrarDllX86)) {
    $missingDlls += "UnRAR.dll (32-bit)"
}
if (($Architecture -eq "Both" -or $Architecture -eq "x64") -and -not (Test-Path $UnrarDllX64)) {
    $missingDlls += "UnRAR64.dll (64-bit)"
}

if ($missingDlls.Count -gt 0) {
    Write-Host "  ⚠ Warning: Missing UnRAR DLLs:" -ForegroundColor Yellow
    foreach ($dll in $missingDlls) {
        Write-Host "    - $dll" -ForegroundColor Yellow
    }
    Write-Host "  RAR/CBR support will not be available in the installer" -ForegroundColor Yellow
} else {
    Write-Host "  ✓ UnRAR DLLs found" -ForegroundColor Green
}

# Step 1: Build Rust project
Write-Host "[1/3] Building Rust project..." -ForegroundColor Yellow
Push-Location CBXShell
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Pop-Location
    throw "Cargo build failed"
}
Pop-Location
Write-Host "  ✓ Rust build completed" -ForegroundColor Green

# Step 2: Create dist directory
Write-Host "[2/3] Preparing distribution directory..." -ForegroundColor Yellow
$DistDir = ".\dist"
if (-not (Test-Path $DistDir)) {
    New-Item -ItemType Directory -Force -Path $DistDir | Out-Null
}
Write-Host "  ✓ Distribution directory ready" -ForegroundColor Green

# Step 3: Build NSIS installers
Write-Host "[3/3] Building NSIS installer(s)..." -ForegroundColor Yellow

$Architectures = @()
if ($Architecture -eq "Both") {
    $Architectures = @("x86", "x64")
} else {
    $Architectures = @($Architecture)
}

$InstallerFiles = @()

foreach ($Arch in $Architectures) {
    Write-Host ""
    Write-Host "  Building $Arch installer..." -ForegroundColor Cyan

    & $NsisPath "/DARCH=$Arch" "installer.nsi"
    if ($LASTEXITCODE -ne 0) {
        throw "NSIS build failed for $Arch"
    }

    $InstallerFile = Get-ChildItem "$DistDir\CBXShell-rs-Setup-*-$Arch.exe" | Select-Object -First 1
    if ($InstallerFile) {
        $InstallerFiles += $InstallerFile
        Write-Host "  ✓ $Arch installer created: $($InstallerFile.Name)" -ForegroundColor Green
        Write-Host "    Size: $([math]::Round($InstallerFile.Length / 1MB, 2)) MB" -ForegroundColor Gray
    } else {
        Write-Host "  ✗ Failed to find $Arch installer output" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "NSIS installer build completed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Output files:" -ForegroundColor Cyan
foreach ($file in $InstallerFiles) {
    Write-Host "  - $($file.FullName)" -ForegroundColor Gray
}
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Test both installers on a clean Windows system" -ForegroundColor Gray
Write-Host "  2. Create a GitHub release and upload both installers" -ForegroundColor Gray
Write-Host "  3. Update release notes with installation instructions" -ForegroundColor Gray
