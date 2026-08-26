param(
    [string]$WindowsArchive = "",
    [string]$WebGLDir = ""
)

$ErrorActionPreference = "Stop"

function Assert-WindowsPackage([string]$Archive) {
    $archivePath = (Resolve-Path -LiteralPath $Archive).Path
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zip = [IO.Compression.ZipFile]::OpenRead($archivePath)
    try {
        $names = @($zip.Entries | ForEach-Object FullName)
        $expected = @("stellar_legacy.exe", "assets.zip")
        $unexpected = @($names | Where-Object { $_ -notin $expected })
        $missing = @($expected | Where-Object { $_ -notin $names })
        if ($missing.Count -or $unexpected.Count) {
            throw "Retail archive mismatch. Missing: $($missing -join ', '); unexpected: $($unexpected -join ', ')"
        }
        $forbidden = '(?i)(\.pdb$|\.env$|steam_appid|password|credential|secret|api[_-]?key)'
        if ($names -match $forbidden) { throw "Retail archive contains a forbidden file name." }
    }
    finally { $zip.Dispose() }

    $tempRoot = [IO.Path]::GetTempPath()
    $work = Join-Path $tempRoot ("stellar-legacy-smoke-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $work | Out-Null
    try {
        Expand-Archive -LiteralPath $archivePath -DestinationPath $work
        $manifest = Join-Path $work "smoke.tsv"
        $frame = Join-Path $work "smoke.png"
        Set-Content -LiteralPath $manifest -Value "menu`t$frame" -Encoding utf8
        $env:STELLAR_LEGACY_CAPTURE_MANIFEST = $manifest
        $env:STELLAR_LEGACY_CAPTURE_FRAMES = "3"
        $env:STELLAR_LEGACY_HEADLESS = "1"
        Push-Location $work
        try { & (Join-Path $work "stellar_legacy.exe") }
        finally { Pop-Location }
        if ($LASTEXITCODE -ne 0) { throw "Packaged executable smoke test exited $LASTEXITCODE." }
        if (-not (Test-Path -LiteralPath $frame) -or (Get-Item -LiteralPath $frame).Length -lt 1000) {
            throw "Packaged executable did not produce a valid capture frame."
        }
    }
    finally {
        Remove-Item Env:STELLAR_LEGACY_CAPTURE_MANIFEST,Env:STELLAR_LEGACY_CAPTURE_FRAMES,Env:STELLAR_LEGACY_HEADLESS -ErrorAction SilentlyContinue
        $resolvedWork = [IO.Path]::GetFullPath($work)
        if ($resolvedWork.StartsWith([IO.Path]::GetFullPath($tempRoot), [StringComparison]::OrdinalIgnoreCase)) {
            Remove-Item -LiteralPath $resolvedWork -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
    Write-Host "Packaged Windows executable launched and rendered successfully." -ForegroundColor Green
}

function Assert-WebGLPackage([string]$Directory) {
    $root = (Resolve-Path -LiteralPath $Directory).Path
    $required = @("index.html", "stellar_legacy.wasm", "assets.zip", "catalog_thumbnail.png")
    foreach ($name in $required) {
        $path = Join-Path $root $name
        if (-not (Test-Path -LiteralPath $path -PathType Leaf) -or (Get-Item $path).Length -eq 0) {
            throw "Packaged WebGL runtime file is missing or empty: $name"
        }
    }
    $magic = [IO.File]::ReadAllBytes((Join-Path $root "stellar_legacy.wasm"))[0..3]
    if (($magic -join ',') -ne '0,97,115,109') { throw "Packaged WebAssembly binary has invalid magic bytes." }
    $html = Get-Content -Raw -LiteralPath (Join-Path $root "index.html")
    if ($html -notmatch 'stellar_legacy\.wasm') {
        throw "Packaged WebGL page does not reference its WebAssembly runtime."
    }

    $browser = @(
        "C:\Program Files\Google\Chrome\Application\chrome.exe",
        "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        "C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        "C:\Program Files\Microsoft\Edge\Application\msedge.exe"
    ) | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
    if (-not $browser) { throw "Chrome or Edge is required for the packaged WebGL runtime smoke test." }

    # The WebGL artifact is deployed beneath the shared Web Hatchery site shell.
    # Recreate that exact relative layout locally so the browser exercises the
    # packaged wasm/assets and the same shared JavaScript loader used in Preview.
    $projectRoot = Split-Path (Split-Path $root -Parent) -Parent
    $workspaceRoot = Split-Path $projectRoot -Parent
    $siteRoot = Join-Path (Split-Path $root -Parent) "webgl-smoke"
    $gameRoot = Join-Path $siteRoot "stellar_legacy"
    $runtimeRoot = Join-Path $siteRoot "shared-assets\runtime"
    $releaseRuntime = Join-Path $workspaceRoot "Release\shared-assets\runtime"
    $managementWeb = Join-Path $workspaceRoot "rust_management\web"
    New-Item -ItemType Directory -Force -Path $gameRoot,$runtimeRoot | Out-Null
    Copy-Item -LiteralPath (Join-Path $root "index.html"),(Join-Path $root "stellar_legacy.wasm"),(Join-Path $root "assets.zip"),(Join-Path $root "catalog_thumbnail.png") -Destination $gameRoot -Force
    Copy-Item -LiteralPath (Join-Path $managementWeb "shared.css"),(Join-Path $managementWeb "bug-report.css"),(Join-Path $managementWeb "bug-report.js") -Destination $siteRoot -Force
    Copy-Item -Path (Join-Path $releaseRuntime "*.js") -Destination $runtimeRoot -Force
    $smokeIndex = Join-Path $gameRoot "index.html"
    $smokeHtml = Get-Content -Raw -LiteralPath $smokeIndex
    $readyProbe = '<script>(function probe(){if(window.wasm_exports){document.body.setAttribute("data-webgl-runtime","ready");}else{setTimeout(probe,50);}})();</script></body>'
    Set-Content -LiteralPath $smokeIndex -Value $smokeHtml.Replace('</body>', $readyProbe) -Encoding utf8

    $listener = [Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback, 0)
    $listener.Start()
    $port = ([Net.IPEndPoint]$listener.LocalEndpoint).Port
    $listener.Stop()
    $token = [guid]::NewGuid().ToString("N")
    $temp = [IO.Path]::GetTempPath()
    $serverOut = Join-Path $temp "stellar-webgl-server-$token.log"
    $serverErr = Join-Path $temp "stellar-webgl-server-$token.err"
    $browserOut = Join-Path $temp "stellar-webgl-browser-$token.log"
    $browserErr = Join-Path $temp "stellar-webgl-browser-$token.err"
    try {
        $server = Start-Process -FilePath "python" -ArgumentList @(
            "-m", "http.server", "$port", "--bind", "127.0.0.1", "--directory", $siteRoot
        ) -RedirectStandardOutput $serverOut -RedirectStandardError $serverErr -WindowStyle Hidden -PassThru
        $ready = $false
        foreach ($attempt in 1..40) {
            try {
                $client = [Net.Sockets.TcpClient]::new("127.0.0.1", $port)
                $client.Dispose()
                $ready = $true
                break
            }
            catch { Start-Sleep -Milliseconds 100 }
        }
        if (-not $ready) { throw "Packaged WebGL test server did not start." }

        $url = "http://127.0.0.1:$port/stellar_legacy/index.html"
        $browserProcess = Start-Process -FilePath $browser -ArgumentList @(
            "--headless=new", "--no-first-run", "--disable-extensions",
            "--enable-unsafe-swiftshader", "--use-angle=swiftshader",
            "--window-size=1280,720", "--virtual-time-budget=12000",
            "--dump-dom", $url
        ) -RedirectStandardOutput $browserOut -RedirectStandardError $browserErr -WindowStyle Hidden -PassThru
        if (-not $browserProcess.WaitForExit(30000)) {
            $browserProcess.Kill()
            throw "Packaged WebGL browser smoke timed out."
        }
        if ($browserProcess.ExitCode -ne 0) { throw "Packaged WebGL browser exited $($browserProcess.ExitCode)." }
        $dom = Get-Content -Raw -LiteralPath $browserOut
        if ($dom -notmatch 'data-webgl-runtime="ready"') {
            Copy-Item -LiteralPath $browserOut -Destination (Join-Path $siteRoot "browser-smoke-failure.html") -Force
            Copy-Item -LiteralPath $browserErr -Destination (Join-Path $siteRoot "browser-smoke-failure.log") -Force
            throw "Packaged WebGL runtime did not initialise; diagnostics preserved in $siteRoot."
        }
    }
    finally {
        if ($server -and -not $server.HasExited) { Stop-Process -Id $server.Id -Force }
        Remove-Item -LiteralPath $serverOut,$serverErr,$browserOut,$browserErr -Force -ErrorAction SilentlyContinue
    }
    Write-Host "Packaged WebGL build loaded and rendered in a real browser." -ForegroundColor Green
}

if ($WindowsArchive) { Assert-WindowsPackage $WindowsArchive }
if ($WebGLDir) { Assert-WebGLPackage $WebGLDir }
