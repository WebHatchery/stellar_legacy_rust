# Start the process directly: Windows PowerShell's Start-Process -PassThru
# can lose the exit code before WaitForExit opens the process handle.
function Invoke-ReleaseBrowser {
    param(
        [string]$FilePath,
        [string]$Arguments,
        [string]$StandardOutput,
        [string]$StandardError,
        [int]$TimeoutMilliseconds = 30000
    )
    $process = New-Object System.Diagnostics.Process
    $process.StartInfo.FileName = $FilePath
    $process.StartInfo.Arguments = $Arguments
    $process.StartInfo.UseShellExecute = $false
    $process.StartInfo.CreateNoWindow = $true
    $process.StartInfo.RedirectStandardOutput = $true
    $process.StartInfo.RedirectStandardError = $true
    $stdout = $null
    $stderr = $null
    try {
        if (-not $process.Start()) { throw "Packaged WebGL browser could not start." }
        # Drain both pipes concurrently so a large diagnostic dump cannot block exit.
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        if (-not $process.WaitForExit($TimeoutMilliseconds)) {
            $process.Kill()
            throw "Packaged WebGL browser smoke timed out."
        }
        if (-not $stdout.Wait(5000) -or -not $stderr.Wait(5000)) {
            throw "Packaged WebGL browser output did not close after exit."
        }
        return $process.ExitCode
    }
    finally {
        if ($stdout -and $stdout.Status -eq 'RanToCompletion') {
            [IO.File]::WriteAllText($StandardOutput, $stdout.Result)
        }
        if ($stderr -and $stderr.Status -eq 'RanToCompletion') {
            [IO.File]::WriteAllText($StandardError, $stderr.Result)
        }
        $process.Dispose()
    }
}
