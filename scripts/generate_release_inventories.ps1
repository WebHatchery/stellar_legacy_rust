$ErrorActionPreference = "Stop"
$project = Split-Path -Parent $PSScriptRoot
Push-Location $project
try {
    $meta = cargo metadata --locked --filter-platform x86_64-pc-windows-msvc --format-version 1 | ConvertFrom-Json -Depth 100
    $root = $meta.packages | Where-Object { $_.manifest_path -eq (Resolve-Path "Cargo.toml").Path } | Select-Object -First 1
    $nodes = @{}; foreach ($node in $meta.resolve.nodes) { $nodes[$node.id] = $node }
    $wanted = [Collections.Generic.HashSet[string]]::new(); $queue = [Collections.Queue]::new(); $queue.Enqueue($root.id)
    while ($queue.Count) {
        $id = [string]$queue.Dequeue()
        if (-not $wanted.Add($id)) { continue }
        foreach ($dep in $nodes[$id].deps) { $queue.Enqueue([string]$dep.pkg) }
    }
    $packages = @($meta.packages | Where-Object { $wanted.Contains($_.id) -and $_.id -ne $root.id } | Sort-Object name,version)
    $lines = @("# Third-party dependency licence inventory", "", "Generated from Cargo.lock for x86_64-pc-windows-msvc. Human notice review remains required.", "", "| Package | Version | Licence | Source |", "| --- | --- | --- | --- |")
    foreach ($package in $packages) {
        $license = if ($package.license) { $package.license } else { "UNKNOWN" }
        $source = if ($package.repository) { $package.repository } elseif ($package.source) { $package.source } else { "local path" }
        $lines += "| $($package.name) | $($package.version) | $license | $source |"
    }
    $lines | Set-Content -LiteralPath "docs\release\THIRD_PARTY_LICENSES.md" -Encoding utf8

    $rows = @('path,category,source,creator,licence_or_permission,ai_assisted,owner_status')
    foreach ($file in Get-ChildItem assets -Recurse -File | Sort-Object FullName) {
        $relative = $file.FullName.Substring($project.Length + 1).Replace('\','/')
        $category = if ($file.Extension -in '.png','.ico') { 'image' } elseif ($relative -match 'dynasty_names|crew_archetypes') { 'name_or_text_corpus' } else { 'game_data_or_text' }
        $rows += '"{0}","{1}","repository","UNKNOWN","UNKNOWN","UNKNOWN","HUMAN REVIEW REQUIRED"' -f $relative,$category
    }
    foreach ($file in Get-ChildItem "docs\release\store_media" -Recurse -File | Sort-Object FullName) {
        $relative = $file.FullName.Substring($project.Length + 1).Replace('\','/')
        $rows += '"{0}","store_media","see STORE_MEDIA_PROVENANCE.md","UNKNOWN","UNKNOWN","YES/INHERITED","HUMAN APPROVAL REQUIRED"' -f $relative
    }
    $rows | Set-Content -LiteralPath "docs\release\ASSET_PROVENANCE.csv" -Encoding utf8
}
finally { Pop-Location }
