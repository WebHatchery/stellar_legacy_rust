param(
    [Parameter(Mandatory)] [string]$Executable,
    [string]$CertificateThumbprint = $env:STELLAR_SIGNING_CERT_THUMBPRINT,
    [string]$TimestampUrl = "http://timestamp.digicert.com"
)

$ErrorActionPreference = "Stop"
if (-not $CertificateThumbprint) { throw "Set STELLAR_SIGNING_CERT_THUMBPRINT in secure local storage." }
$signtool = Get-Command signtool.exe -ErrorAction Stop
& $signtool.Source sign /sha1 $CertificateThumbprint /fd SHA256 /tr $TimestampUrl /td SHA256 $Executable
if ($LASTEXITCODE -ne 0) { throw "Authenticode signing failed." }
& $signtool.Source verify /pa /all /v $Executable
if ($LASTEXITCODE -ne 0) { throw "Authenticode verification failed." }
