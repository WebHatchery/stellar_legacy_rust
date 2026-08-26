param(
    [switch]$AllowDirty,
    [switch]$ScanWithDefender
)

$ErrorActionPreference = "Stop"
$project = Split-Path -Parent $PSScriptRoot
Push-Location $project
try {
    $dirty = [bool](git status --porcelain)
    if ($dirty -and -not $AllowDirty) { throw "Release candidates must be built from a clean commit." }
    $metadata = cargo metadata --no-deps --format-version 1 | ConvertFrom-Json
    $version = $metadata.packages |
        Where-Object { $_.manifest_path -eq (Resolve-Path "Cargo.toml").Path } |
        Select-Object -ExpandProperty version -First 1
    & .\publish.ps1 -WindowsOnly -DryRun
    if (-not $?) { throw "Windows publisher failed." }

    $source = (Resolve-Path "dist\stellar_legacy_windows.zip").Path
    $releaseDir = Join-Path $project "dist\releases"
    New-Item -ItemType Directory -Path $releaseDir -Force | Out-Null
    $artifact = Join-Path $releaseDir "stellar_legacy_${version}_windows_x86_64.zip"
    Copy-Item -LiteralPath $source -Destination $artifact -Force

    $scanStarted = $null
    $detectionCount = $null
    if ($ScanWithDefender) {
        $scanStarted = [DateTime]::UtcNow
        Start-MpScan -ScanType CustomScan -ScanPath $artifact
        $detections = @(Get-MpThreatDetection -ErrorAction Stop | Where-Object {
            $_.InitialDetectionTime.ToUniversalTime() -ge $scanStarted -and
                (($_.Resources -join "`n") -match [regex]::Escape($artifact))
        })
        $detectionCount = $detections.Count
        if ($detectionCount -gt 0) {
            throw "Windows Defender reported $detectionCount detection(s) for the release candidate."
        }
    }
    $toolkit = Join-Path (Split-Path $project -Parent) "macroquad-toolkit"
    $manifest = [ordered]@{
        product = "Stellar Legacy"; version = $version; platform = "windows-x86_64"
        commit = (git rev-parse HEAD); dirty = $dirty
        rust = (rustc --version); toolkit_commit = (git -C $toolkit rev-parse HEAD)
        built_utc = [DateTime]::UtcNow.ToString("o")
        artifact = [IO.Path]::GetFileName($artifact)
        bytes = (Get-Item $artifact).Length
        sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $artifact).Hash.ToLowerInvariant()
        cargo_lock_sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $metadata.workspace_root "Cargo.lock")).Hash.ToLowerInvariant()
        defender_scan_requested = [bool]$ScanWithDefender
        defender_scan_started_utc = if ($scanStarted) { $scanStarted.ToString("o") } else { $null }
        defender_detection_count = $detectionCount
    }
    $manifest | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $releaseDir "release-manifest.json") -Encoding utf8
    $manifest.GetEnumerator() | ForEach-Object { "{0}: {1}" -f $_.Key, $_.Value } |
        Set-Content -LiteralPath (Join-Path $releaseDir "release-manifest.txt") -Encoding utf8
    Write-Host "Release candidate: $artifact" -ForegroundColor Green
}
finally { Pop-Location }
