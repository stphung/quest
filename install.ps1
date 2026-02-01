# One-line installer for Quest (Windows)
# Usage: irm https://raw.githubusercontent.com/stphung/quest/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "stphung/quest"
$Binary = "quest.exe"
$Target = "x86_64-pc-windows-msvc"
$InstallDir = if ($env:QUEST_INSTALL_DIR) { $env:QUEST_INSTALL_DIR } else { "$env:USERPROFILE\.local\bin" }

Write-Host "Detected platform: $Target"
Write-Host "Fetching latest release..."

$ReleaseUrl = "https://api.github.com/repos/$Repo/releases/latest"
$Release = Invoke-RestMethod -Uri $ReleaseUrl -Headers @{ "User-Agent" = "quest-installer" }

$Asset = $Release.assets | Where-Object { $_.name -like "*$Target*" } | Select-Object -First 1
if (-not $Asset) {
    Write-Error "Could not find a release for $Target. Check https://github.com/$Repo/releases"
    exit 1
}

Write-Host "Found: $($Asset.browser_download_url)"

$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $TmpDir | Out-Null

try {
    $ZipPath = Join-Path $TmpDir "quest.zip"

    Write-Host "Downloading..."
    Invoke-WebRequest -Uri $Asset.browser_download_url -OutFile $ZipPath

    Write-Host "Extracting..."
    Expand-Archive -Path $ZipPath -DestinationPath $TmpDir

    Write-Host "Installing to $InstallDir..."
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Copy-Item -Path (Join-Path $TmpDir $Binary) -Destination (Join-Path $InstallDir $Binary) -Force
}
finally {
    Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
}

# Check if install dir is in PATH
$CurrentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($CurrentPath -notlike "*$InstallDir*") {
    Write-Host ""
    Write-Host "Quest has been installed to $InstallDir\$Binary"
    Write-Host ""
    Write-Host "Add $InstallDir to your PATH by running:"
    Write-Host ""
    Write-Host "  [Environment]::SetEnvironmentVariable('PATH', `"$InstallDir;`$env:PATH`", 'User')"
    Write-Host ""
}
else {
    Write-Host ""
    Write-Host "Quest has been installed! Run 'quest' to start playing."
}
