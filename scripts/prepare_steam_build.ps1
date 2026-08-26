param(
    [Parameter(Mandatory)] [string]$AppId,
    [Parameter(Mandatory)] [string]$DepotId,
    [string]$BuildLabel = "rc",
    [switch]$Upload,
    [string]$TestBranch = ""
)

$ErrorActionPreference = "Stop"
$project = Split-Path -Parent $PSScriptRoot
$archive = Join-Path $project "dist\stellar_legacy_windows.zip"
& (Join-Path $PSScriptRoot "test_release_package.ps1") -WindowsArchive $archive

$stage = Join-Path $project "dist\steam\content"
$config = Join-Path $project "dist\steam\config"
if (Test-Path $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
New-Item -ItemType Directory -Path $stage,$config -Force | Out-Null
Expand-Archive -LiteralPath $archive -DestinationPath $stage

$version = (cargo metadata --manifest-path (Join-Path $project "Cargo.toml") --no-deps --format-version 1 | ConvertFrom-Json).packages |
    Where-Object { $_.manifest_path -eq (Resolve-Path (Join-Path $project "Cargo.toml")).Path } |
    Select-Object -ExpandProperty version -First 1
$replacements = @{
    "{{APP_ID}}" = $AppId; "{{DEPOT_ID}}" = $DepotId; "{{VERSION}}" = $version
    "{{BUILD_LABEL}}" = $BuildLabel; "{{BUILD_OUTPUT}}" = (Join-Path $project "dist\steam\output")
    "{{CONTENT_ROOT}}" = $stage
}
foreach ($name in "app_build","depot_build") {
    $text = Get-Content -Raw -LiteralPath (Join-Path $project "steam\${name}.vdf.template")
    foreach ($entry in $replacements.GetEnumerator()) { $text = $text.Replace($entry.Key, $entry.Value) }
    Set-Content -LiteralPath (Join-Path $config "${name}.vdf") -Value $text -Encoding utf8
}
Write-Host "Steam content staged. Launch option: stellar_legacy.exe" -ForegroundColor Green
Write-Host "Preview config: $(Join-Path $config 'app_build.vdf')"

if ($Upload) {
    if (-not $TestBranch) { throw "Uploads require an explicit password-protected test-branch name." }
    $steamcmd = Get-Command steamcmd.exe -ErrorAction Stop
    Write-Host "Human-authorised upload to test branch '$TestBranch'. Credentials remain in SteamCMD." -ForegroundColor Yellow
    & $steamcmd.Source +login +run_app_build (Join-Path $config "app_build.vdf") +quit
    if ($LASTEXITCODE -ne 0) { throw "SteamCMD upload failed." }
    Write-Host "Record the returned Build ID, then assign it to '$TestBranch' in Steamworks." -ForegroundColor Yellow
}
