# RustGames project publisher wrapper.
# Build/deploy behavior lives in the workspace root publish.ps1.

param(
    [switch]$SkipBuild = $false,
    [switch]$WindowsOnly = $false,
    [switch]$WebGLOnly = $false,
    [switch]$DeployOnly = $false,
    [Alias('p')] [switch]$Production = $false,
    [switch]$FTP = $false,
    [switch]$DryRun = $false
)

$ErrorActionPreference = "Stop"
$rootPublisher = Join-Path (Split-Path $PSScriptRoot -Parent) "publish.ps1"

if (-not (Test-Path $rootPublisher)) {
    Write-Error "RustGames root publisher not found: $rootPublisher"
    exit 1
}

& $rootPublisher -RustGamePublish -ProjectDir $PSScriptRoot `
    -SkipBuild:$SkipBuild `
    -WindowsOnly:$WindowsOnly `
    -WebGLOnly:$WebGLOnly `
    -DeployOnly:$DeployOnly `
    -Production:$Production `
    -FTP:$FTP `
    -DryRun:$DryRun

if (-not $?) { exit 1 }

if (-not $DeployOnly) {
    $smoke = Join-Path $PSScriptRoot "scripts\test_release_package.ps1"
    $smokeArgs = @{}
    if (-not $WebGLOnly) {
        $smokeArgs.WindowsArchive = Join-Path $PSScriptRoot "dist\stellar_legacy_windows.zip"
    }
    if (-not $WindowsOnly) {
        $smokeArgs.WebGLDir = Join-Path $PSScriptRoot "dist\webgl"
    }
    & $smoke @smokeArgs
    if (-not $?) { exit 1 }
}
