<#
.SYNOPSIS
    Headless screenshot harness for Feast Frenzy.

.DESCRIPTION
    Thin wrapper around the shared macroquad-toolkit capture script. Builds the
    debug exe and drives it through the env-var capture hook
    (feast_FRENZY_CAPTURE_*) provided by macroquad_toolkit::capture in
    src/app.rs (invoked from src/main.rs). Scenes map to App::begin_capture_scene:
    "title" (main menu), "settings" (settings screen), and "gameplay" (default,
    starts a fresh game and jumps into the kitchen).

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Scenes gameplay -Frames 60 -SkipBuild
#>
param(
    [string[]]$Scenes = @("title", "settings", "gameplay"),
    [int]$Frames = 150,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Prefix "feast_FRENZY" -Scenes $Scenes -Frames $Frames -OutputDir $OutputDir -SkipBuild:$SkipBuild
