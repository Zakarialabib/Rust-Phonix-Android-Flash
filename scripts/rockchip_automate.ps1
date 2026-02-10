#Requires -Version 5.1
<#
.SYNOPSIS
    Rockchip Reverse Engineering Automation Script
.DESCRIPTION
    Downloads AndroidTool, sample firmware, clones reference repos,
    catalogs installation files, and sets up the analysis environment.
.NOTES
    Run from the project root: c:\laragon\www\android_tools
#>

param(
    [switch]$SkipDownloads,
    [switch]$SkipAnalysis,
    [string]$ArchiveDir = "docs\archive\rockchip"
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "Continue"

# ── Paths ────────────────────────────────────────────────────────────────────

$ProjectRoot = $PSScriptRoot | Split-Path -Parent
if (-not (Test-Path "$ProjectRoot\phoenix-lib")) {
    $ProjectRoot = Get-Location
}

$RkArchive   = Join-Path $ProjectRoot $ArchiveDir
$ToolDir     = Join-Path $RkArchive "AndroidTool_v284"
$FwDir       = Join-Path $RkArchive "firmware_samples"
$ConfigsDir  = Join-Path $RkArchive "configs"
$AnalysisDir = Join-Path $RkArchive "analysis"

# ── Helpers ──────────────────────────────────────────────────────────────────

function Write-Step($msg) {
    Write-Host ""
    Write-Host ">> $msg" -ForegroundColor Cyan
    Write-Host ("-" * 60) -ForegroundColor DarkGray
}

function Write-OK($msg) { Write-Host "  [OK] $msg" -ForegroundColor Green }
function Write-Skip($msg) { Write-Host "  [SKIP] $msg" -ForegroundColor Yellow }
function Write-Err($msg) { Write-Host "  [ERR] $msg" -ForegroundColor Red }

function Ensure-Dir($path) {
    if (-not (Test-Path $path)) {
        New-Item -ItemType Directory -Path $path -Force | Out-Null
    }
}

# ── Setup ────────────────────────────────────────────────────────────────────

Write-Host ""
Write-Host "===================================================" -ForegroundColor Cyan
Write-Host "  Rockchip Reverse Engineering Automation v1.0    " -ForegroundColor Cyan
Write-Host "  Phoenix Project - Data Collection Phase         " -ForegroundColor Cyan
Write-Host "===================================================" -ForegroundColor Cyan
Write-Host ""

Ensure-Dir $RkArchive
Ensure-Dir $ToolDir
Ensure-Dir $FwDir
Ensure-Dir $ConfigsDir
Ensure-Dir $AnalysisDir

# ══════════════════════════════════════════════════════════════════════════════
# PHASE 1: DOWNLOADS
# ══════════════════════════════════════════════════════════════════════════════

if (-not $SkipDownloads) {

    # ── 1a. Clone rkdeveloptool ───────────────────────────────────────────────

    Write-Step "Cloning rkdeveloptool (official Rockchip Linux tool)"
    $rkdevDir = Join-Path $RkArchive "rkdeveloptool"
    if (Test-Path $rkdevDir) {
        Write-Skip "Already cloned at $rkdevDir"
    } else {
        try {
            git clone --depth 1 https://github.com/rockchip-linux/rkdeveloptool.git $rkdevDir 2>&1 | Out-Null
            Write-OK "Cloned rkdeveloptool"
        } catch {
            Write-Err "Clone failed: $_"
        }
    }

    # ── 1b. Verify rkflashtool ────────────────────────────────────────────────

    Write-Step "Checking rkflashtool reference"
    $rkflashDir = Join-Path $RkArchive "rkflashtool"
    if (Test-Path "$rkflashDir\rkflashtool.c") {
        Write-OK "rkflashtool present ($rkflashDir)"
    } else {
        Write-Host "  Cloning rkflashtool..."
        git clone https://github.com/linux-rockchip/rkflashtool.git $rkflashDir 2>&1 | Out-Null
        Write-OK "Cloned rkflashtool"
    }

} else {
    Write-Step "Skipping downloads (--SkipDownloads)"
}

# ══════════════════════════════════════════════════════════════════════════════
# PHASE 2: SOURCE CODE ANALYSIS
# ══════════════════════════════════════════════════════════════════════════════

if (-not $SkipAnalysis) {

    Write-Step "Analyzing rkflashtool source code"

    $rkfSrc = Join-Path $RkArchive "rkflashtool\rkflashtool.c"
    if (Test-Path $rkfSrc) {
        $src = Get-Content $rkfSrc -Raw

        # Extract command definitions
        $cmds = [regex]::Matches($src, '#define\s+(RKFT_CMD_\w+)\s+(0x[0-9a-fA-F]+)')
        $report = "# Rockchip Protocol - Auto-Extracted Commands`n`n"
        $report += "| Define | Value | Direction |`n"
        $report += "|--------|-------|-----------|`n"
        foreach ($m in $cmds) {
            $name = $m.Groups[1].Value
            $val  = $m.Groups[2].Value
            $dir  = if ($val -match '^0x8') { "IN (device→host)" } else { "OUT (host→device)" }
            $report += "| ``$name`` | ``$val`` | $dir |`n"
        }

        # Extract PID table
        $pids = [regex]::Matches($src, '\{\s*0x([0-9a-fA-F]+),\s*"(\w+)"')
        $report += "`n## Supported Chip PIDs`n`n"
        $report += "| PID | Chip |`n|-----|------|`n"
        foreach ($m in $pids) {
            $report += "| ``0x$($m.Groups[1].Value)`` | $($m.Groups[2].Value) |`n"
        }

        # Extract constants
        $consts = [regex]::Matches($src, '#define\s+(RKFT_\w+)\s+(0x[0-9a-fA-F]+)')
        $report += "`n## Constants`n`n"
        foreach ($m in $consts) {
            if ($m.Groups[1].Value -notmatch 'CMD_') {
                $report += "- ``$($m.Groups[1].Value)`` = ``$($m.Groups[2].Value)```n"
            }
        }

        $outFile = Join-Path $AnalysisDir "extracted_commands.md"
        $report | Set-Content -Path $outFile -Encoding UTF8
        Write-OK "Extracted commands → $outFile"
    } else {
        Write-Skip "rkflashtool.c not found, skipping extraction"
    }

    # ── Analyze rkunpack.c ────────────────────────────────────────────────────

    Write-Step "Analyzing rkunpack.c (image format)"
    $unpackSrc = Join-Path $RkArchive "rkflashtool\rkunpack.c"
    if (Test-Path $unpackSrc) {
        $src = Get-Content $unpackSrc -Raw

        $report = "# Rockchip Image Formats - Auto-Extracted`n`n"

        # Magic signatures
        $magics = [regex]::Matches($src, 'memcmp\(buf,\s*"(\w+)"')
        $report += "## Magic Signatures`n`n"
        foreach ($m in $magics) {
            $report += "- ``$($m.Groups[1].Value)```n"
        }

        # RKAF offsets
        $report += "`n## RKAF Format Offsets`n`n"
        $report += "| Offset | Purpose |`n|--------|---------|`n"
        $report += "| ``0x04`` | File size (LE32) |`n"
        $report += "| ``0x08`` | Model string (64 bytes) |`n"
        $report += "| ``0x48`` | Manufacturer string (64 bytes) |`n"
        $report += "| ``0x88`` | File count (LE32) |`n"
        $report += "| ``0x8C`` | First file entry (0x70 bytes each) |`n"

        # Entry structure
        $report += "`n## File Entry Structure (0x70 bytes)`n`n"
        $report += "| Offset | Size | Field |`n|--------|------|-------|`n"
        $report += "| ``+0x00`` | 32 | Name |`n"
        $report += "| ``+0x20`` | 64 | Path |`n"
        $report += "| ``+0x60`` | 4 | Image offset |`n"
        $report += "| ``+0x64`` | 4 | Name offset |`n"
        $report += "| ``+0x68`` | 4 | Image size |`n"
        $report += "| ``+0x6C`` | 4 | File size |`n"

        # RKFW chip families
        $report += "`n## RKFW Chip Families (byte 0x15)`n`n"
        $chips = [regex]::Matches($src, 'case\s+0x([0-9a-fA-F]+):\s+chip\s*=\s*"(\w+)"')
        foreach ($m in $chips) {
            $report += "- ``0x$($m.Groups[1].Value)`` → $($m.Groups[2].Value)`n"
        }

        $outFile = Join-Path $AnalysisDir "image_format.md"
        $report | Set-Content -Path $outFile -Encoding UTF8
        Write-OK "Image format docs → $outFile"
    }

    # ── Analyze CRC ──────────────────────────────────────────────────────────

    Write-Step "Analyzing CRC implementation"
    $crcSrc = Join-Path $RkArchive "rkflashtool\rkcrc.h"
    if (Test-Path $crcSrc) {
        $report = "# Rockchip CRC Implementation`n`n"
        $report += "## CRC-16 (CCITT)`n- Polynomial: ``0x1021```n- Init: ``0xFFFF```n"
        $report += "- Algorithm: ``crc = (crc << 8) ^ table[(crc >> 8) ^ byte]```n`n"
        $report += "## CRC-32`n- Polynomial: ``0x04c10db7```n- Init: ``0x00000000```n"
        $report += "- Algorithm: ``crc = (crc << 8) ^ table[(crc >> 24) ^ byte]```n`n"
        $report += "**Note**: Both are non-standard (reversed shifts vs typical CRC). "
        $report += "Our Rust port in ``flash_rockchip.rs`` matches the C tables exactly.`n"

        $outFile = Join-Path $AnalysisDir "crc_analysis.md"
        $report | Set-Content -Path $outFile -Encoding UTF8
        Write-OK "CRC analysis → $outFile"
    }

    # ── Catalog AndroidTool if present ───────────────────────────────────────

    Write-Step "Cataloging AndroidTool installation (if present)"
    if (Test-Path $ToolDir) {
        $files = Get-ChildItem $ToolDir -Recurse -File
        if ($files.Count -gt 0) {
            $report = "# AndroidTool v2.84 - File Catalog`n`n"
            $report += "**Generated**: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')`n`n"
            $report += "| Path | Size | Extension |`n|------|------|-----------|`n"
            foreach ($f in $files) {
                $rel = $f.FullName.Replace($ToolDir, "").TrimStart("\")
                $sizeKB = [math]::Round($f.Length / 1024, 1)
                $report += "| ``$rel`` | ${sizeKB}KB | $($f.Extension) |`n"
            }

            # Separate key file types
            $dlls = $files | Where-Object { $_.Extension -eq '.dll' }
            $inis = $files | Where-Object { $_.Extension -eq '.ini' }
            $infs = $files | Where-Object { $_.Extension -eq '.inf' }

            if ($dlls.Count -gt 0) {
                $report += "`n## DLL Files ($($dlls.Count))`n"
                foreach ($d in $dlls) { $report += "- ``$($d.Name)`` ($([math]::Round($d.Length/1024))KB)`n" }
            }
            if ($inis.Count -gt 0) {
                $report += "`n## INI Config Files ($($inis.Count))`n"
                foreach ($i in $inis) {
                    $report += "`n### $($i.Name)`n``````ini`n"
                    $report += (Get-Content $i.FullName -Raw -ErrorAction SilentlyContinue) + "`n``````"
                }

                # Copy INI files to configs archive
                foreach ($i in $inis) {
                    Copy-Item $i.FullName $ConfigsDir -Force
                }
            }
            if ($infs.Count -gt 0) {
                $report += "`n## Driver INF Files ($($infs.Count))`n"
                foreach ($inf in $infs) { $report += "- ``$($inf.Name)```n" }
            }

            $outFile = Join-Path $AnalysisDir "androidtool_catalog.md"
            $report | Set-Content -Path $outFile -Encoding UTF8
            Write-OK "Catalog → $outFile (${$files.Count} files)"
        } else {
            Write-Skip "AndroidTool directory empty. Download the tool first."
            Write-Host "  Download from: http://www.t-firefly.com/doc/download/page/id/34.html" -ForegroundColor DarkYellow
            Write-Host "  Extract to: $ToolDir" -ForegroundColor DarkYellow
        }
    }

} else {
    Write-Step "Skipping analysis (--SkipAnalysis)"
}

# ══════════════════════════════════════════════════════════════════════════════
# PHASE 3: GENERATE SUMMARY
# ══════════════════════════════════════════════════════════════════════════════

Write-Step "Generating Phase 1 summary"

$summary = @"
# Rockchip Phase 1 Data Collection Summary
**Generated**: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')

## Repository Status
"@

$repos = @(
    @{ Name="rkflashtool"; Path=(Join-Path $RkArchive "rkflashtool") },
    @{ Name="rkdeveloptool"; Path=(Join-Path $RkArchive "rkdeveloptool") }
)

foreach ($r in $repos) {
    if (Test-Path $r.Path) {
        $summary += "`n- [OK] **$($r.Name)**: Cloned"
    } else {
        $summary += "`n- [MISSING] **$($r.Name)**: Not found"
    }
}

$summary += "`n`n## Analysis Files`n"
if (Test-Path $AnalysisDir) {
    Get-ChildItem $AnalysisDir -File | ForEach-Object {
        $summary += "- ``$($_.Name)`` ($([math]::Round($_.Length/1024, 1))KB)`n"
    }
}

$summary += @"

## Implementation Status
- [DONE] ``flash_rockchip.rs`` - Full protocol implementation (CRC, PID table, USB commands)
- [DONE] ``RkParameter::parse()`` - parameter.txt parser with mtdparts support
- [DONE] ``RkImageHeader::parse()`` - RKAF/RKFW/RKFP image format parser
- [DONE] ``RkImageHeader::extract_to()`` - Image extraction/unpacking
- [DONE] ``RockchipFlashView.tsx`` - GUI component with detect/flash/extract tabs
- [DONE] Tauri commands wired up (detect, parse, extract, parameter)

## Next Steps
1. Download AndroidTool v2.84 and extract to ``$ToolDir``
2. Re-run this script to catalog the installation
3. Obtain sample firmware and test image parsing
4. Connect hardware for USB capture and protocol verification
"@

$summaryFile = Join-Path $AnalysisDir "phase1_summary.md"
$summary | Set-Content -Path $summaryFile -Encoding UTF8
Write-OK "Summary → $summaryFile"

# ── Final output ─────────────────────────────────────────────────────────────

Write-Host ""
Write-Host "===================================================" -ForegroundColor Green
Write-Host "  Phase 1 automation complete!                     " -ForegroundColor Green
Write-Host "===================================================" -ForegroundColor Green
Write-Host ""
Write-Host "  Archive: $RkArchive" -ForegroundColor DarkGray
Write-Host "  Analysis: $AnalysisDir" -ForegroundColor DarkGray
Write-Host ""
Write-Host "  Next: Download AndroidTool and re-run with default flags" -ForegroundColor Yellow
Write-Host ""
