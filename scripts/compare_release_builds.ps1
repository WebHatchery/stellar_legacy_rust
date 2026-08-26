param(
    [switch]$AllowDirty
)

$ErrorActionPreference = "Stop"
$project = Split-Path -Parent $PSScriptRoot
$tempRoot = [IO.Path]::GetTempPath()
$work = Join-Path $tempRoot ("stellar release comparison " + [guid]::NewGuid().ToString("N"))

function Get-ZipPayload([string]$Archive) {
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zip = [IO.Compression.ZipFile]::OpenRead($Archive)
    try {
        $entries = [ordered]@{}
        foreach ($entry in $zip.Entries | Sort-Object FullName) {
            $stream = $entry.Open()
            try {
                $sha = [Security.Cryptography.SHA256]::Create()
                try { $hash = [Convert]::ToHexString($sha.ComputeHash($stream)).ToLowerInvariant() }
                finally { $sha.Dispose() }
            }
            finally { $stream.Dispose() }
            $entries[$entry.FullName] = [ordered]@{ bytes = $entry.Length; sha256 = $hash }
        }
        return $entries
    }
    finally { $zip.Dispose() }
}

Push-Location $project
try {
    if ((git status --porcelain) -and -not $AllowDirty) {
        throw "Repeat-build comparison requires a clean commit."
    }
    New-Item -ItemType Directory -Path $work | Out-Null
    $copies = @()
    foreach ($pass in 1..2) {
        & .\publish.ps1 -WindowsOnly -DryRun
        if (-not $?) { throw "Release build $pass failed." }
        $copy = Join-Path $work "stellar_legacy_build_$pass.zip"
        Copy-Item -LiteralPath "dist\stellar_legacy_windows.zip" -Destination $copy
        $copies += $copy
    }

    $firstPayload = Get-ZipPayload $copies[0]
    $secondPayload = Get-ZipPayload $copies[1]
    $payloadEqual = ($firstPayload | ConvertTo-Json -Depth 5 -Compress) -ceq
        ($secondPayload | ConvertTo-Json -Depth 5 -Compress)
    if (-not $payloadEqual) { throw "Packaged runtime payload changed between builds." }

    $firstArchiveHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $copies[0]).Hash.ToLowerInvariant()
    $secondArchiveHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $copies[1]).Hash.ToLowerInvariant()
    $report = [ordered]@{
        compared_utc = [DateTime]::UtcNow.ToString("o")
        commit = (git rev-parse HEAD)
        archive_byte_identical = $firstArchiveHash -ceq $secondArchiveHash
        first_archive_sha256 = $firstArchiveHash
        second_archive_sha256 = $secondArchiveHash
        payload_identical = $payloadEqual
        payload = $firstPayload
        note = "Payload identity is the release gate; ZIP metadata may make container hashes differ."
    }
    $releaseDir = Join-Path $project "dist\releases"
    New-Item -ItemType Directory -Path $releaseDir -Force | Out-Null
    $report | ConvertTo-Json -Depth 7 | Set-Content -LiteralPath (Join-Path $releaseDir "build-comparison.json") -Encoding utf8
    Write-Host "Two release builds have identical packaged payloads." -ForegroundColor Green
    Write-Host "ZIP bytes identical: $($report.archive_byte_identical)"
}
finally {
    Pop-Location
    $resolvedWork = [IO.Path]::GetFullPath($work)
    if ($resolvedWork.StartsWith([IO.Path]::GetFullPath($tempRoot), [StringComparison]::OrdinalIgnoreCase)) {
        Remove-Item -LiteralPath $resolvedWork -Recurse -Force -ErrorAction SilentlyContinue
    }
}
