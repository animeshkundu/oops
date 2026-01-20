# oops installer script for Windows

param(
    [string]$Version = "latest",
    [string]$InstallDir = "$env:LOCALAPPDATA\oops"
)

$ErrorActionPreference = "Stop"

Write-Host "Installing oops for Windows..."

# Create install directory
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

# Get download URL
$BinaryName = "oops-windows-x86_64.exe"
if ($Version -eq "latest") {
    $DownloadUrl = "https://github.com/oops-cli/oops/releases/latest/download/$BinaryName"
} else {
    $DownloadUrl = "https://github.com/oops-cli/oops/releases/download/$Version/$BinaryName"
}

# Download
Write-Host "Downloading from $DownloadUrl..."
$TempFile = Join-Path $env:TEMP "oops.exe"
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempFile

# Install
$DestPath = Join-Path $InstallDir "oops.exe"
Move-Item -Path $TempFile -Destination $DestPath -Force

Write-Host ""
Write-Host "oops installed to: $DestPath"
Write-Host ""

# Add to PATH if not already there
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($CurrentPath -notlike "*$InstallDir*") {
    Write-Host "Adding to PATH..."
    [Environment]::SetEnvironmentVariable(
        "Path",
        "$CurrentPath;$InstallDir",
        "User"
    )
    Write-Host "PATH updated. Please restart your terminal."
}

Write-Host ""
Write-Host "To complete setup, add to your PowerShell profile:"
Write-Host ""
Write-Host '  Invoke-Expression (oops --alias | Out-String)'
Write-Host ""
Write-Host "Profile location: $PROFILE"
Write-Host ""
Write-Host "You can edit it with: notepad `$PROFILE"
