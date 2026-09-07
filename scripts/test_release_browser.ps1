$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'release_browser.ps1')

$shellPath = (Get-Process -Id $PID).Path
$token = [guid]::NewGuid().ToString('N')
$stdoutPath = Join-Path ([IO.Path]::GetTempPath()) "stellar-browser-test-$token.out"
$stderrPath = Join-Path ([IO.Path]::GetTempPath()) "stellar-browser-test-$token.err"
try {
    # Exercise fast exits and enough output on both pipes to expose deadlocks.
    foreach ($expected in @(0, 7, 0)) {
        $command = "[Console]::Out.Write(('o' * 100000)); [Console]::Error.Write(('e' * 100000)); exit $expected"
        $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($command))
        $actual = Invoke-ReleaseBrowser -FilePath $shellPath `
            -Arguments "-NoProfile -EncodedCommand $encoded" `
            -StandardOutput $stdoutPath -StandardError $stderrPath
        if ($null -eq $actual -or $actual -ne $expected) {
            throw "Exit code mismatch: expected $expected, got '$actual'."
        }
        if ([IO.File]::ReadAllText($stdoutPath).Length -ne 100000 -or
            [IO.File]::ReadAllText($stderrPath).Length -ne 100000) {
            throw 'The process output was not drained completely.'
        }
    }
    $timedOut = $false
    try {
        $null = Invoke-ReleaseBrowser -FilePath $shellPath `
            -Arguments '-NoProfile -Command "Start-Sleep -Seconds 10"' `
            -StandardOutput $stdoutPath -StandardError $stderrPath -TimeoutMilliseconds 100
    }
    catch {
        if ($_.Exception.Message -notmatch 'timed out') { throw }
        $timedOut = $true
    }
    if (-not $timedOut) { throw 'The timeout did not terminate the process.' }
    Write-Host "Browser process regression checks passed in PowerShell $($PSVersionTable.PSVersion)."
}
finally {
    Remove-Item -LiteralPath $stdoutPath,$stderrPath -Force -ErrorAction SilentlyContinue
}
