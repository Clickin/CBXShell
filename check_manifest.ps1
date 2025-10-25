# Quick script to check MSIX manifest architecture

param([string]$MsixPath)

Add-Type -AssemblyName System.IO.Compression.FileSystem

$zip = [System.IO.Compression.ZipFile]::OpenRead($MsixPath)
$entry = $zip.Entries | Where-Object { $_.Name -eq 'AppxManifest.xml' }
$stream = $entry.Open()
$reader = New-Object System.IO.StreamReader($stream)
$content = $reader.ReadToEnd()
$reader.Close()
$zip.Dispose()

$xml = [xml]$content
$identity = $xml.Package.Identity

Write-Host "Package: $(Split-Path -Leaf $MsixPath)" -ForegroundColor Cyan
Write-Host "  Name: $($identity.Name)" -ForegroundColor Gray
Write-Host "  Publisher: $($identity.Publisher)" -ForegroundColor Gray
Write-Host "  Architecture: $($identity.ProcessorArchitecture)" -ForegroundColor Yellow
Write-Host "  Version: $($identity.Version)" -ForegroundColor Gray
Write-Host ""
