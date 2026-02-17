# Authy install script — Windows (PowerShell)
# Usage: irm https://raw.githubusercontent.com/eric8810/authy/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$Repo = "eric8810/authy"
$Binary = "authy"
$Target = "x86_64-pc-windows-msvc"

function Main {
    $version = Get-Version
    Download-And-Install $version
    Verify-Install
}

function Get-Version {
    if ($env:AUTHY_VERSION) {
        Write-Host "Using specified version: $env:AUTHY_VERSION" -ForegroundColor Green
        return $env:AUTHY_VERSION
    }

    Write-Host "Fetching latest version..." -ForegroundColor Green
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    $version = $release.tag_name

    if (-not $version) {
        throw "Failed to determine latest version"
    }

    Write-Host "Latest version: $version" -ForegroundColor Green
    return $version
}

function Download-And-Install($version) {
    $archive = "$Binary-$Target.zip"
    $url = "https://github.com/$Repo/releases/download/$version/$archive"

    $tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
    New-Item -ItemType Directory -Path $tmpDir | Out-Null

    try {
        Write-Host "Downloading $url..." -ForegroundColor Green
        Invoke-WebRequest -Uri $url -OutFile (Join-Path $tmpDir $archive) -UseBasicParsing

        Write-Host "Extracting..." -ForegroundColor Green
        Expand-Archive -Path (Join-Path $tmpDir $archive) -DestinationPath $tmpDir

        $installDir = Join-Path $env:LOCALAPPDATA "authy"
        if (-not (Test-Path $installDir)) {
            New-Item -ItemType Directory -Path $installDir | Out-Null
        }

        Copy-Item -Path (Join-Path $tmpDir "$Binary.exe") -Destination (Join-Path $installDir "$Binary.exe") -Force
        Write-Host "Installed to $installDir\$Binary.exe" -ForegroundColor Green

        # Add to User PATH if not present
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($userPath -notlike "*$installDir*") {
            [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
            $env:Path = "$env:Path;$installDir"
            Write-Host "Added $installDir to User PATH" -ForegroundColor Green
        }
    }
    finally {
        Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
    }
}

function Verify-Install {
    $exe = Join-Path $env:LOCALAPPDATA "authy\$Binary.exe"
    if (Test-Path $exe) {
        try {
            $ver = & $exe --version 2>&1
            Write-Host "Verification: $ver" -ForegroundColor Green
        }
        catch {
            Write-Host "Install complete. Restart your terminal to use '$Binary'." -ForegroundColor Green
        }
    }
}

Main
