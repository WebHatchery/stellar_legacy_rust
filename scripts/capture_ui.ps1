<#
.SYNOPSIS
    Headless screenshot harness for the game template.

.DESCRIPTION
    Thin wrapper around the shared macroquad-toolkit capture script. Builds the
    debug executable and photographs named Stellar Legacy verification scenes.

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Frames 60 -SkipBuild
#>
param(
    [string[]]$Scenes = @("gameplay"),
    [int]$Frames = 150,
    [int]$WindowWidth = 0,
    [int]$WindowHeight = 0,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Prefix "STELLAR_LEGACY" -Scenes $Scenes -Frames $Frames `
    -WindowWidth $WindowWidth -WindowHeight $WindowHeight -OutputDir $OutputDir `
    -SkipBuild:$SkipBuild
