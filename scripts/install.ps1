#Requires -Version 5.1
<#
.SYNOPSIS
    Download and install md2cb from GitHub.

.DESCRIPTION
    This script downloads the latest release or main branch build of md2cb
    and installs it to the specified directory.

.PARAMETER FromMain
    Download the latest successful build from the main branch instead of
    the latest release. Requires GitHub CLI (gh) to be installed and authenticated.

.PARAMETER InstallDir
    Installation directory. Defaults to ~/bin (or ~\bin on Windows).

.EXAMPLE
    .\install.ps1
    Install the latest release to ~/bin

.EXAMPLE
    .\install.ps1 -FromMain
    Install the latest build from main branch

.EXAMPLE
    .\install.ps1 -InstallDir "C:\Tools"
    Install to a custom directory
#>

[CmdletBinding()]
param(
    [Alias("m")]
    [switch]$FromMain,

    [Alias("d")]
    [string]$InstallDir
)

$ErrorActionPreference = "Stop"

$Repo = "letientai299/md2cb"
$BinaryName = "md2cb"

function Get-Platform {
    $os = if ($IsWindows -or $env:OS -eq "Windows_NT") {
        "windows"
    } elseif ($IsLinux) {
        "linux"
    } elseif ($IsMacOS) {
        "macos"
    } else {
        throw "Unsupported operating system"
    }

    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    $archStr = switch ($arch) {
        "X64" { "x64" }
        "Arm64" {
            if ($os -eq "macos") { "arm64" }
            else { throw "ARM64 is only supported on macOS" }
        }
        default { throw "Unsupported architecture: $arch" }
    }

    return "$os-$archStr"
}

$Platform = Get-Platform
$IsWindowsPlatform = $Platform -like "windows-*"
$ArtifactName = "md2cb-$Platform"
$ArchiveExt = if ($IsWindowsPlatform) { "zip" } else { "tar.gz" }
$BinaryExt = if ($IsWindowsPlatform) { ".exe" } else { "" }

# Set default install dir based on platform
if (-not $InstallDir) {
    $InstallDir = if ($IsWindowsPlatform) {
        "$env:USERPROFILE\bin"
    } else {
        "$env:HOME/.local/bin"
    }
}

function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] $Message" -ForegroundColor Red
    exit 1
}

function Get-LatestRelease {
    Write-Info "Fetching latest release..."

    $releaseUrl = "https://api.github.com/repos/$Repo/releases/latest"
    $release = Invoke-RestMethod -Uri $releaseUrl -UseBasicParsing

    $assetPattern = "*$Platform.$ArchiveExt"
    $asset = $release.assets | Where-Object { $_.name -like $assetPattern } | Select-Object -First 1

    if (-not $asset) {
        Write-Error "Could not find release artifact for platform: $Platform"
    }

    $downloadUrl = $asset.browser_download_url
    $tempFile = Join-Path ([System.IO.Path]::GetTempPath()) "$ArtifactName.$ArchiveExt"

    Write-Info "Downloading $($asset.name)..."
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -UseBasicParsing

    return $tempFile
}

function Get-FromMain {
    Write-Info "Fetching latest successful build from main branch..."

    # Check if gh CLI is available
    $ghPath = Get-Command gh -ErrorAction SilentlyContinue
    if (-not $ghPath) {
        Write-Error "GitHub CLI (gh) is required for -FromMain. Install from: https://cli.github.com/"
    }

    # Check if authenticated
    $authStatus = gh auth status 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Error "GitHub CLI not authenticated. Run: gh auth login"
    }

    # Get latest successful workflow run
    $runJson = gh run list --repo $Repo --branch main --workflow ci.yml --status success --limit 1 --json databaseId 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to fetch workflow runs: $runJson"
    }

    $runs = $runJson | ConvertFrom-Json
    if (-not $runs -or $runs.Count -eq 0) {
        Write-Error "No successful workflow runs found on main branch"
    }

    $runId = $runs[0].databaseId
    Write-Info "Found workflow run: $runId"

    $tempDir = Join-Path ([System.IO.Path]::GetTempPath()) "md2cb-download-$PID"
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

    Write-Info "Downloading artifact: $ArtifactName..."
    gh run download $runId --repo $Repo --name $ArtifactName --dir $tempDir

    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to download artifact"
    }

    # Find the downloaded archive
    $archive = Get-ChildItem -Path $tempDir -Filter "*.$ArchiveExt" -Recurse | Select-Object -First 1

    if (-not $archive) {
        Write-Error "Could not find downloaded archive in $tempDir"
    }

    return $archive.FullName
}

function Install-Binary {
    param(
        [string]$Archive,
        [string]$DestDir
    )

    Write-Info "Extracting archive..."

    $tempExtract = Join-Path ([System.IO.Path]::GetTempPath()) "md2cb-extract-$PID"
    New-Item -ItemType Directory -Path $tempExtract -Force | Out-Null

    if ($Archive -like "*.tar.gz") {
        # Use tar for .tar.gz files (available on all modern platforms)
        tar -xzf $Archive -C $tempExtract
    } else {
        Expand-Archive -Path $Archive -DestinationPath $tempExtract -Force
    }

    # Find the binary
    $binaryFileName = "$BinaryName$BinaryExt"
    $binary = Get-ChildItem -Path $tempExtract -Filter $binaryFileName -Recurse | Select-Object -First 1

    if (-not $binary) {
        Write-Error "Could not find $binaryFileName in archive"
    }

    # Create install directory if needed
    if (-not (Test-Path $DestDir)) {
        Write-Info "Creating directory: $DestDir"
        New-Item -ItemType Directory -Path $DestDir -Force | Out-Null
    }

    $destPath = Join-Path $DestDir $binaryFileName
    Write-Info "Installing to $destPath..."
    Copy-Item -Path $binary.FullName -Destination $destPath -Force

    # Make executable on Unix
    if (-not $IsWindowsPlatform) {
        chmod +x $destPath
    }

    # Cleanup
    Remove-Item -Path $tempExtract -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -Path $Archive -Force -ErrorAction SilentlyContinue

    Write-Info "Successfully installed $BinaryName to $destPath"

    # Check if directory is in PATH
    $pathSeparator = if ($IsWindowsPlatform) { ";" } else { ":" }
    $pathDirs = $env:PATH -split $pathSeparator
    if ($pathDirs -notcontains $DestDir) {
        Write-Warn "$DestDir is not in your PATH"
        if ($IsWindowsPlatform) {
            Write-Warn "Add it with: `$env:PATH += `";$DestDir`""
            Write-Warn "Or permanently: [Environment]::SetEnvironmentVariable('PATH', `$env:PATH + ';$DestDir', 'User')"
        } else {
            Write-Warn "Add it with: export PATH=`"`$PATH:$DestDir`""
        }
    }
}

# Main
try {
    Write-Info "Platform: $Platform"

    $archive = if ($FromMain) {
        Get-FromMain
    } else {
        Get-LatestRelease
    }

    Install-Binary -Archive $archive -DestDir $InstallDir

} catch {
    Write-Error $_.Exception.Message
}
